#!/usr/bin/env python3
"""
OPURE OS - Standalone UEFI 128MB Virtual Disk Image Generator
Generates a valid FAT32 EFI System Partition containing /EFI/BOOT/BOOTX64.EFI
"""

import os
import struct
import sys

DISK_SIZE_BYTES = 128 * 1024 * 1024 # 128 MB
SECTOR_SIZE = 512
TOTAL_SECTORS = DISK_SIZE_BYTES // SECTOR_SIZE # 262,144 sectors
SECTORS_PER_CLUSTER = 8 # 4096 bytes per cluster
RESERVED_SECTORS = 32
NUM_FATS = 2
SECTORS_PER_FAT = 256

def make_fat_name(name, ext=""):
    name_bytes = name.upper().encode('ascii')[:8].ljust(8, b' ')
    ext_bytes = ext.upper().encode('ascii')[:3].ljust(3, b' ')
    return name_bytes + ext_bytes

def create_dir_entry(name, ext, attr, cluster, size):
    entry = bytearray(32)
    entry[0:11] = make_fat_name(name, ext)
    entry[11] = attr
    # time & date
    entry[14:16] = struct.pack('<H', 0x4821)
    entry[16:18] = struct.pack('<H', 0x58A4)
    entry[20:22] = struct.pack('<H', (cluster >> 16) & 0xFFFF)
    entry[26:28] = struct.pack('<H', cluster & 0xFFFF)
    entry[28:32] = struct.pack('<I', size)
    return entry

def package_disk(efi_binary_path, output_image_path):
    print(f"[PACKAGE] Reading EFI binary: {efi_binary_path}")
    if not os.path.exists(efi_binary_path):
        raise FileNotFoundError(f"EFI binary not found at {efi_binary_path}")
    
    with open(efi_binary_path, 'rb') as f:
        efi_data = f.read()

    efi_size = len(efi_data)
    print(f"[PACKAGE] EFI binary size: {efi_size} bytes")

    cluster_size = SECTOR_SIZE * SECTORS_PER_CLUSTER # 4096 bytes
    efi_clusters_needed = (efi_size + cluster_size - 1) // cluster_size

    # Cluster allocation:
    # Cluster 2: Root Directory
    # Cluster 3: /EFI Directory
    # Cluster 4: /EFI/BOOT Directory
    # Cluster 5 .. 5 + efi_clusters_needed - 1: /EFI/BOOT/BOOTX64.EFI
    efi_start_cluster = 5

    # Prepare disk buffer (128 MB)
    disk = bytearray(DISK_SIZE_BYTES)

    # 1. Boot Sector (LBA 0)
    boot = bytearray(SECTOR_SIZE)
    boot[0:3] = b'\xeb\x58\x90'
    boot[3:11] = b'MSWIN4.1'
    boot[11:13] = struct.pack('<H', SECTOR_SIZE)
    boot[13] = SECTORS_PER_CLUSTER
    boot[14:16] = struct.pack('<H', RESERVED_SECTORS)
    boot[16] = NUM_FATS
    boot[17:19] = struct.pack('<H', 0)
    boot[19:21] = struct.pack('<H', 0)
    boot[21] = 0xF8 # Fixed disk
    boot[22:24] = struct.pack('<H', 0)
    boot[24:26] = struct.pack('<H', 63)
    boot[26:28] = struct.pack('<H', 255)
    boot[28:32] = struct.pack('<I', 0) # Hidden sectors
    boot[32:36] = struct.pack('<I', TOTAL_SECTORS)
    boot[36:40] = struct.pack('<I', SECTORS_PER_FAT)
    boot[40:42] = struct.pack('<H', 0)
    boot[42:44] = struct.pack('<H', 0)
    boot[44:48] = struct.pack('<I', 2) # Root cluster 2
    boot[48:50] = struct.pack('<H', 1) # FSInfo sector 1
    boot[50:52] = struct.pack('<H', 6) # Backup boot sector 6
    boot[64] = 0x80 # Drive number
    boot[66] = 0x29 # Extended boot signature
    boot[67:71] = struct.pack('<I', 0x50555245) # Vol ID: "PURE"
    boot[71:82] = b'OPURE_EFI  '
    boot[82:90] = b'FAT32   '
    boot[510:512] = b'\x55\xaa'
    disk[0:SECTOR_SIZE] = boot
    # Backup boot sector at sector 6
    disk[6*SECTOR_SIZE : 7*SECTOR_SIZE] = boot

    # 2. FSInfo Sector (LBA 1)
    fsinfo = bytearray(SECTOR_SIZE)
    fsinfo[0:4] = b'RRaA'
    fsinfo[484:488] = b'rrAa'
    free_clusters = (TOTAL_SECTORS - RESERVED_SECTORS - (NUM_FATS * SECTORS_PER_FAT)) // SECTORS_PER_CLUSTER - (4 + efi_clusters_needed)
    fsinfo[488:492] = struct.pack('<I', free_clusters)
    fsinfo[492:496] = struct.pack('<I', efi_start_cluster + efi_clusters_needed)
    fsinfo[508:512] = b'\x00\x00\x55\xaa'
    disk[1*SECTOR_SIZE : 2*SECTOR_SIZE] = fsinfo
    disk[7*SECTOR_SIZE : 8*SECTOR_SIZE] = fsinfo

    # 3. FAT Tables
    fat1_offset = RESERVED_SECTORS * SECTOR_SIZE
    fat2_offset = (RESERVED_SECTORS + SECTORS_PER_FAT) * SECTOR_SIZE

    def set_fat_entry(cluster, value):
        offset = cluster * 4
        struct.pack_into('<I', disk, fat1_offset + offset, value)
        struct.pack_into('<I', disk, fat2_offset + offset, value)

    # Reserved entries
    set_fat_entry(0, 0x0FFFFFF8)
    set_fat_entry(1, 0xFFFFFFFF)
    # Cluster 2 (root dir) -> EOC
    set_fat_entry(2, 0x0FFFFFFF)
    # Cluster 3 (/EFI) -> EOC
    set_fat_entry(3, 0x0FFFFFFF)
    # Cluster 4 (/EFI/BOOT) -> EOC
    set_fat_entry(4, 0x0FFFFFFF)

    # Cluster chain for BOOTX64.EFI
    for i in range(efi_clusters_needed):
        curr_cluster = efi_start_cluster + i
        if i == efi_clusters_needed - 1:
            set_fat_entry(curr_cluster, 0x0FFFFFFF) # EOC
        else:
            set_fat_entry(curr_cluster, curr_cluster + 1)

    # Helper to calculate cluster byte offset
    data_region_lba = RESERVED_SECTORS + (NUM_FATS * SECTORS_PER_FAT)
    def cluster_to_byte_offset(cluster):
        return (data_region_lba + (cluster - 2) * SECTORS_PER_CLUSTER) * SECTOR_SIZE

    # 4. Root Directory (Cluster 2)
    # Contains: "EFI" [ATTR_DIRECTORY = 0x10] -> Cluster 3
    root_offset = cluster_to_byte_offset(2)
    efi_entry = create_dir_entry("EFI", "", 0x10, 3, 0)
    disk[root_offset : root_offset + 32] = efi_entry

    # 5. /EFI Directory (Cluster 3)
    # Contains: "." (Cluster 3), ".." (Cluster 0/2), "BOOT" (Cluster 4)
    efi_dir_offset = cluster_to_byte_offset(3)
    dot_entry = create_dir_entry(".", "", 0x10, 3, 0)
    dotdot_entry = create_dir_entry("..", "", 0x10, 0, 0)
    boot_entry = create_dir_entry("BOOT", "", 0x10, 4, 0)
    disk[efi_dir_offset : efi_dir_offset + 32] = dot_entry
    disk[efi_dir_offset + 32 : efi_dir_offset + 64] = dotdot_entry
    disk[efi_dir_offset + 64 : efi_dir_offset + 96] = boot_entry

    # 6. /EFI/BOOT Directory (Cluster 4)
    # Contains: "." (Cluster 4), ".." (Cluster 3), "BOOTX64.EFI" (Cluster 5)
    boot_dir_offset = cluster_to_byte_offset(4)
    b_dot = create_dir_entry(".", "", 0x10, 4, 0)
    b_dotdot = create_dir_entry("..", "", 0x10, 3, 0)
    bootx64_entry = create_dir_entry("BOOTX64", "EFI", 0x20, efi_start_cluster, efi_size)
    disk[boot_dir_offset : boot_dir_offset + 32] = b_dot
    disk[boot_dir_offset + 32 : boot_dir_offset + 64] = b_dotdot
    disk[boot_dir_offset + 64 : boot_dir_offset + 96] = bootx64_entry

    # 7. Write BOOTX64.EFI contents (starting at Cluster 5)
    file_offset = cluster_to_byte_offset(efi_start_cluster)
    disk[file_offset : file_offset + efi_size] = efi_data

    # Write out disk image
    print(f"[PACKAGE] Writing 128MB EFI virtual disk container: {output_image_path}")
    with open(output_image_path, 'wb') as f:
        f.write(disk)

    print(f"[PACKAGE] SUCCESS: Created {output_image_path} ({len(disk)} bytes)")
    print(f"[PACKAGE] Structure: /EFI/BOOT/BOOTX64.EFI verified.")

if __name__ == '__main__':
    root_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    efi_path = os.path.join(root_dir, 'target', 'x86_64-unknown-uefi', 'release', 'opure_boot.efi')
    output_path = os.path.join(root_dir, 'opure_disk.img')
    package_disk(efi_path, output_path)
