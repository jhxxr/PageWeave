import os
from PIL import Image, ImageDraw


SOURCE_ICON_PATH = r"C:\Users\24717\.gemini\antigravity\brain\3ac3f65e-eec6-4623-912f-67896b70b056\pageweave_logo_1783388737642.png"
TARGET_DIR = r"g:\0JHX-code\Project\PageWeave\src-tauri\icons"
PUBLIC_DIR = r"g:\0JHX-code\Project\PageWeave\public"
OUTER_MARGIN_RATIO = 0.06
CORNER_RADIUS_RATIO = 0.18


def crop_outer_margin(img):
    w, h = img.size
    margin_w = int(w * OUTER_MARGIN_RATIO)
    margin_h = int(h * OUTER_MARGIN_RATIO)
    return img.crop((margin_w, margin_h, w - margin_w, h - margin_h))


def apply_rounded_border_mask(img):
    img = img.convert("RGBA")
    w, h = img.size
    scale = 4
    radius = int(min(w, h) * CORNER_RADIUS_RATIO)
    large_size = (w * scale, h * scale)
    large_radius = radius * scale

    mask = Image.new("L", large_size, 0)
    draw = ImageDraw.Draw(mask)
    draw.rounded_rectangle(
        (0, 0, large_size[0] - 1, large_size[1] - 1),
        radius=large_radius,
        fill=255,
    )
    mask = mask.resize((w, h), Image.Resampling.LANCZOS)

    rounded = Image.new("RGBA", img.size, (0, 0, 0, 0))
    rounded.paste(img, (0, 0), mask)
    return rounded

def generate_icons():
    if not os.path.exists(SOURCE_ICON_PATH):
        print(f"Error: Source image not found at {SOURCE_ICON_PATH}")
        return
        
    print(f"Loading source icon: {SOURCE_ICON_PATH}")
    img = Image.open(SOURCE_ICON_PATH)
    w, h = img.size
    print(f"Original size: {w}x{h}")

    cropped_img = apply_rounded_border_mask(crop_outer_margin(img))
    print(f"Cropped size (center 88%): {cropped_img.size[0]}x{cropped_img.size[1]}")
    
    # Ensure directories exist
    os.makedirs(TARGET_DIR, exist_ok=True)
    os.makedirs(PUBLIC_DIR, exist_ok=True)
    
    # Save the public logo for web UI
    logo_path = os.path.join(PUBLIC_DIR, "logo.png")
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
        dest_path = os.path.join(TARGET_DIR, filename)
        resized = cropped_img.resize((size, size), Image.Resampling.LANCZOS)
        resized.save(dest_path, "PNG")
        print(f"Generated {filename} ({size}x{size})")
        
    # Generate Windows icon.ico (contains multiple sizes)
    ico_sizes = [16, 32, 48, 64, 128, 256]
    ico_path = os.path.join(TARGET_DIR, "icon.ico")
    cropped_img.save(ico_path, format="ICO", sizes=[(sz, sz) for sz in ico_sizes])
    print(f"Generated icon.ico (sizes: {ico_sizes})")
    
    # Generate macOS icon.icns
    icns_path = os.path.join(TARGET_DIR, "icon.icns")
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
