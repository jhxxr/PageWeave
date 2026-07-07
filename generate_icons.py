import os
from PIL import Image

def generate_icons():
    # Source image path
    src_path = r"C:\Users\24717\.gemini\antigravity\brain\3ac3f65e-eec6-4623-912f-67896b70b056\pageweave_logo_1783388737642.png"
    target_dir = r"g:\0JHX-code\Project\PageWeave\src-tauri\icons"
    public_dir = r"g:\0JHX-code\Project\PageWeave\public"
    
    if not os.path.exists(src_path):
        print(f"Error: Source image not found at {src_path}")
        return
        
    print(f"Loading source icon: {src_path}")
    img = Image.open(src_path)
    w, h = img.size
    print(f"Original size: {w}x{h}")
    
    # Crop the image: remove outer 6% margin from each side (keep center 88%)
    # This removes empty borders and crops the icon to make it look full and premium.
    margin_w = int(w * 0.06)
    margin_h = int(h * 0.06)
    left = margin_w
    top = margin_h
    right = w - margin_w
    bottom = h - margin_h
    
    cropped_img = img.crop((left, top, right, bottom))
    print(f"Cropped size (center 88%): {cropped_img.size[0]}x{cropped_img.size[1]}")
    
    # Ensure directories exist
    os.makedirs(target_dir, exist_ok=True)
    os.makedirs(public_dir, exist_ok=True)
    
    # Save the public logo for web UI
    logo_path = os.path.join(public_dir, "logo.png")
    cropped_img.resize((256, 256), Image.Resampling.LANCZOS).save(logo_path, "PNG")
    print(f"Saved public logo: {logo_path}")
    
    # Tauri PNG icons mapping: name -> size
    png_icons = {
        "32x32.png": 32,
        "128x128.png": 128,
        "128x128@2x.png": 256,
        "icon.png": 512,
        "Square30x30Logo.png": 30,
        "Square44x44Logo.png": 44,
        "Square71x71Logo.png": 71,
        "Square89x89Logo.png": 89,
        "Square107x107Logo.png": 107,
        "Square142x142Logo.png": 142,
        "Square150x150Logo.png": 150,
        "Square284x284Logo.png": 284,
        "Square310x310Logo.png": 310,
        "StoreLogo.png": 50,
    }
    
    for filename, size in png_icons.items():
        dest_path = os.path.join(target_dir, filename)
        resized = cropped_img.resize((size, size), Image.Resampling.LANCZOS)
        resized.save(dest_path, "PNG")
        print(f"Generated {filename} ({size}x{size})")
        
    # Generate Windows icon.ico (contains multiple sizes)
    ico_sizes = [16, 32, 48, 64, 128, 256]
    ico_images = [cropped_img.resize((sz, sz), Image.Resampling.LANCZOS) for sz in ico_sizes]
    ico_path = os.path.join(target_dir, "icon.ico")
    ico_images[0].save(ico_path, format="ICO", append_images=ico_images[1:])
    print(f"Generated icon.ico (sizes: {ico_sizes})")
    
    # Generate macOS icon.icns
    icns_path = os.path.join(target_dir, "icon.icns")
    try:
        # Pillow supports saving ICNS format
        cropped_img.save(icns_path, format="ICNS")
        print(f"Generated icon.icns successfully")
    except Exception as e:
        print(f"Warning: Could not save ICNS using PIL direct save: {e}")
        # Fallback to saving a high-res PNG renamed as icns or let it fail gracefully
        # Standard PIL supports ICNS format on most platforms.
        
    print("Icon generation completed successfully!")

if __name__ == "__main__":
    generate_icons()
