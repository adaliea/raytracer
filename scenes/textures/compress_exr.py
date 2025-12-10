import os
import sys
import argparse
import shutil
import OpenEXR
import Imath

def convert_single_file(input_path, output_path, compression=OpenEXR.ZIP_COMPRESSION):
    """
    Reads an EXR, sets compression, and writes to output_path.
    Safe for overwriting (uses a temp file strategy if input == output).
    """
    try:
        # Check if we are overwriting the file in place
        is_overwrite = os.path.abspath(input_path) == os.path.abspath(output_path)

        # If overwriting, write to a temp file first
        write_path = output_path + ".tmp" if is_overwrite else output_path

        # Ensure output directory exists
        output_dir = os.path.dirname(write_path)
        if not os.path.exists(output_dir):
            os.makedirs(output_dir)

        # 1. Open Input
        if not os.path.exists(input_path):
            print(f"Error: Input file missing: {input_path}")
            return False

        infile = OpenEXR.InputFile(input_path)
        header = infile.header()

        # 2. Update Compression
        header['compression'] = compression

        # 3. Read Channels
        channels_header = header['channels']
        channel_data = {}
        for channel_name, channel_info in channels_header.items():
            channel_data[channel_name] = infile.channel(channel_name, channel_info.type)

        # 4. Write Output
        outfile = OpenEXR.OutputFile(write_path, header)
        outfile.writePixels(channel_data)
        outfile.close()
        infile.close()

        # 5. Handle Overwrite/Renaming
        if is_overwrite:
            # Remove original and rename temp to original
            os.remove(input_path)
            os.rename(write_path, input_path)

        print(f"[OK] {input_path} -> {output_path}")
        return True

    except Exception as e:
        print(f"[FAIL] {input_path}: {e}")
        # Cleanup temp file if it exists and we failed
        if 'write_path' in locals() and os.path.exists(write_path) and is_overwrite:
            os.remove(write_path)
        return False

def process_directory(input_root, output_root, overwrite=False):
    """
    Walks through input_root, matches structure in output_root (if not overwriting),
    and converts files.
    """
    success_count = 0
    fail_count = 0

    print(f"Scanning: {input_root}")
    print(f"Recursive mode: ON")
    print("-" * 40)

    for root, dirs, files in os.walk(input_root):
        for filename in files:
            if filename.lower().endswith('.exr'):
                input_path = os.path.join(root, filename)

                # Calculate relative path to maintain folder structure
                rel_path = os.path.relpath(input_path, input_root)

                if overwrite:
                    output_path = input_path
                else:
                    # Construct mirrored path in output directory
                    output_path = os.path.join(output_root, rel_path)

                if convert_single_file(input_path, output_path):
                    success_count += 1
                else:
                    fail_count += 1

    print("-" * 40)
    print(f"Batch Complete. Converted: {success_count} | Failed: {fail_count}")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Batch convert EXR files to ZIP compression recursively.")

    parser.add_argument("input", help="Path to the folder containing EXR files.")
    parser.add_argument("-o", "--output", help="Path to output folder. (Optional if --overwrite is used)")
    parser.add_argument("--overwrite", action="store_true", help="Overwrite original files in place.")

    args = parser.parse_args()

    input_folder = args.input
    output_folder = args.output

    # Validation
    if not os.path.exists(input_folder):
        print(f"Error: Input path does not exist: {input_folder}")
        sys.exit(1)

    if args.overwrite:
        print("WARNING: You have selected to overwrite files in place.")
        confirm = input("Are you sure? (y/n): ")
        if confirm.lower() != 'y':
            print("Operation cancelled.")
            sys.exit(0)
        output_folder = input_folder # Just for logic consistency
    else:
        # If no output specified and no overwrite, create a default side-folder
        if not output_folder:
            output_folder = os.path.normpath(input_folder) + "_zip"
            print(f"No output folder specified. Defaulting to: {output_folder}")

    process_directory(input_folder, output_folder, args.overwrite)