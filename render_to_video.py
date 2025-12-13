import os
import re
import subprocess
import glob

def get_framerate_from_scene(scene_name, scenes_dir="./scenes", default_framerate=10):
    """
    Parses the scene file to find the frameRate.
    """
    scene_file_path = os.path.join(scenes_dir, f"{scene_name}.ray")
    if not os.path.exists(scene_file_path):
        print(f"Scene file not found: {scene_file_path}")
        return default_framerate

    try:
        with open(scene_file_path, "r") as f:
            content = f.read()
            # Use regex to find "frameRate" followed by a number.
            # This is more robust than simple string splitting.
            match = re.search(r"frameRate\s+(\d+)", content)
            if match:
                return int(match.group(1))
            else:
                print(f"frameRate not found in {scene_file_path}, using default.")
                return default_framerate
    except Exception as e:
        print(f"Error reading scene file {scene_file_path}: {e}")
        return default_framerate


def create_videos_from_renders(output_dir="./output"):
    """
    Looks for rendered scenes in the specified output directory and converts them into videos using ffmpeg.
    Assumes images are named scene_name_frame_number.png and are in subdirectories named scene_name.
    """
    if not os.path.exists(output_dir):
        print(f"Output directory '{output_dir}' does not exist.")
        return

    scene_dirs = [d for d in os.listdir(output_dir) if os.path.isdir(os.path.join(output_dir, d))]

    if not scene_dirs:
        print(f"No scene directories found in '{output_dir}'.")
        return

    for scene_name in scene_dirs:
        framerate = get_framerate_from_scene(scene_name)
        scene_path = os.path.join(output_dir, scene_name)
        
        # Process main output (denoised, no suffix)
        input_pattern_main = os.path.join(scene_path, f"{scene_name}_*.png")
        images_main = sorted(glob.glob(input_pattern_main))
        
        final_images = [img for img in images_main if os.path.basename(img).startswith(f"{scene_name}_") and not any(s in os.path.basename(img) for s in ["_albedo_", "_normal_", "_noisy_"])]
        
        if final_images:
            output_video_path = os.path.join(output_dir, f"{scene_name}.mp4")
            ffmpeg_command = [
                "ffmpeg",
                "-y",
                "-i", os.path.join(scene_path, f"{scene_name}_%d.png"), 
                "-r", str(framerate),
                "-pix_fmt", "yuv420p",
                "-crf", "23",
                output_video_path
            ]
            print(f"Creating video for scene '{scene_name}' (main output) with framerate {framerate}...")
            try:
                subprocess.run(ffmpeg_command, check=True, capture_output=True, text=True)
                print(f"Successfully created video: {output_video_path}")
            except subprocess.CalledProcessError as e:
                print(f"Error creating video for '{scene_name}':")
                print(f"STDOUT: {e.stdout}")
                print(f"STDERR: {e.stderr}")
            except FileNotFoundError:
                print("Error: ffmpeg not found. Please ensure ffmpeg is installed and in your PATH.")
        else:
            print(f"No main output images found for scene '{scene_name}' with pattern '{scene_name}_*.png'.")

        # Process noisy output
        input_pattern_noisy = os.path.join(scene_path, f"{scene_name}_noisy_*.png")
        images_noisy = sorted(glob.glob(input_pattern_noisy))
        if images_noisy:
            output_video_path_noisy = os.path.join(output_dir, f"{scene_name}_noisy.mp4")
            ffmpeg_command_noisy = [
                "ffmpeg",
                "-y",
                "-i", os.path.join(scene_path, f"{scene_name}_noisy_%d.png"),
                "-r", str(framerate),
                "-pix_fmt", "yuv420p",
                "-crf", "23",
                output_video_path_noisy
            ]
            print(f"Creating video for scene '{scene_name}' (noisy output) with framerate {framerate}...")
            try:
                subprocess.run(ffmpeg_command_noisy, check=True, capture_output=True, text=True)
                print(f"Successfully created video: {output_video_path_noisy}")
            except subprocess.CalledProcessError as e:
                print(f"Error creating noisy video for '{scene_name}':")
                print(f"STDOUT: {e.stdout}")
                print(f"STDERR: {e.stderr}")
            except FileNotFoundError:
                print("Error: ffmpeg not found. Please ensure ffmpeg is installed and in your PATH.")
        else:
            print(f"No noisy output images found for scene '{scene_name}' with pattern '{scene_name}_noisy_*.png'.")

        # Process albedo output
        input_pattern_albedo = os.path.join(scene_path, f"{scene_name}_albedo_*.png")
        images_albedo = sorted(glob.glob(input_pattern_albedo))
        if images_albedo:
            output_video_path_albedo = os.path.join(output_dir, f"{scene_name}_albedo.mp4")
            ffmpeg_command_albedo = [
                "ffmpeg",
                "-y",
                "-i", os.path.join(scene_path, f"{scene_name}_albedo_%d.png"),
                "-r", str(framerate),
                "-pix_fmt", "yuv420p",
                "-crf", "23",
                output_video_path_albedo
            ]
            print(f"Creating video for scene '{scene_name}' (albedo output) with framerate {framerate}...")
            try:
                subprocess.run(ffmpeg_command_albedo, check=True, capture_output=True, text=True)
                print(f"Successfully created video: {output_video_path_albedo}")
            except subprocess.CalledProcessError as e:
                print(f"Error creating albedo video for '{scene_name}':")
                print(f"STDOUT: {e.stdout}")
                print(f"STDERR: {e.stderr}")
            except FileNotFoundError:
                print("Error: ffmpeg not found. Please ensure ffmpeg is installed and in your PATH.")
        else:
            print(f"No albedo output images found for scene '{scene_name}' with pattern '{scene_name}_albedo_*.png'.")

        # Process normal output
        input_pattern_normal = os.path.join(scene_path, f"{scene_name}_normal_*.png")
        images_normal = sorted(glob.glob(input_pattern_normal))
        if images_normal:
            output_video_path_normal = os.path.join(output_dir, f"{scene_name}_normal.mp4")
            ffmpeg_command_normal = [
                "ffmpeg",
                "-y",
                "-i", os.path.join(scene_path, f"{scene_name}_normal_%d.png"),
                "-r", str(framerate),
                "-pix_fmt", "yuv420p",
                "-crf", "23",
                output_video_path_normal
            ]
            print(f"Creating video for scene '{scene_name}' (normal output) with framerate {framerate}...")
            try:
                subprocess.run(ffmpeg_command_normal, check=True, capture_output=True, text=True)
                print(f"Successfully created video: {output_video_path_normal}")
            except subprocess.CalledProcessError as e:
                print(f"Error creating normal video for '{scene_name}':")
                print(f"STDOUT: {e.stdout}")
                print(f"STDERR: {e.stderr}")
            except FileNotFoundError:
                print("Error: ffmpeg not found. Please ensure ffmpeg is installed and in your PATH.")
        else:
            print(f"No normal output images found for scene '{scene_name}' with pattern '{scene_name}_normal_*.png'.")


if __name__ == "__main__":
    create_videos_from_renders()
