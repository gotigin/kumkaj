#!/usr/bin/env python3
"""Pack a staged Windows payload into an MSIX (AppxManifest + BlockMap + zip).

Usage:
  pack_msix.py --payload DIR --out FILE.msix \\
    --name Identity.Name --publisher "CN=..." --display NAME \\
    --version 0.1.0.0 --exe kumkaj-app.exe --description TEXT \\
    [--icon-png PATH] [--publisher-display NAME]
"""
from __future__ import annotations

import argparse
import hashlib
import os
import shutil
import struct
import sys
import tempfile
import zipfile
from pathlib import Path
from xml.sax.saxutils import escape

BLOCK = 64 * 1024

# Files that belong in MSI/folder packages but not in the Store payload.
SKIP_NAMES = {
    "Setup.bat",
    "Uninstall.bat",
    "install.ps1",
    "uninstall.ps1",
    "package.wxs",
    "License.rtf",
    "Quick start.txt",
    "README.txt",
    "msi-post-install.js",
    "msi-uninstall-data.js",
    "desktop-ids.sh",
    "windows-wipe-prompt.ps1",
    "delete-books-data.ps1",
}


def four_part_version(v: str) -> str:
    parts = [p for p in v.strip().split(".") if p != ""]
    if len(parts) == 3:
        parts.append("0")
    if len(parts) != 4:
        raise SystemExit(f"version must be a.b.c or a.b.c.d, got {v!r}")
    for p in parts:
        if not p.isdigit() or int(p) > 65535:
            raise SystemExit(f"bad version component in {v!r}")
    return ".".join(str(int(p)) for p in parts)


def write_logos(assets: Path, icon_png: Path | None) -> None:
    assets.mkdir(parents=True, exist_ok=True)
    try:
        from PIL import Image
    except ImportError as e:
        raise SystemExit("Pillow required for MSIX logos (python-pillow)") from e

    if icon_png is None or not icon_png.is_file():
        raise SystemExit(f"icon PNG missing: {icon_png}")

    src = Image.open(icon_png).convert("RGBA")

    def save(name: str, size: tuple[int, int], *, letterbox: bool = False) -> None:
        if letterbox:
            canvas = Image.new("RGBA", size, (0, 0, 0, 0))
            scale = min(size[0] / src.width, size[1] / src.height)
            w = max(1, int(src.width * scale))
            h = max(1, int(src.height * scale))
            im = src.resize((w, h), Image.Resampling.LANCZOS)
            canvas.paste(im, ((size[0] - w) // 2, (size[1] - h) // 2), im)
            canvas.save(assets / name, format="PNG")
        else:
            src.resize(size, Image.Resampling.LANCZOS).save(assets / name, format="PNG")

    save("StoreLogo.png", (50, 50))
    save("Square44x44Logo.png", (44, 44))
    save("Square150x150Logo.png", (150, 150))
    save("Wide310x150Logo.png", (310, 150), letterbox=True)
    save("SplashScreen.png", (620, 300), letterbox=True)


def write_manifest(
    path: Path,
    *,
    name: str,
    publisher: str,
    display: str,
    publisher_display: str,
    version: str,
    exe: str,
    description: str,
) -> None:
    desc = description or display
    # packagedClassicApp + mediumIL: real AppData (same as MSI), not containerized.
    xml = f"""<?xml version="1.0" encoding="utf-8"?>
<Package
  xmlns="http://schemas.microsoft.com/appx/manifest/foundation/windows10"
  xmlns:uap="http://schemas.microsoft.com/appx/manifest/uap/windows10"
  xmlns:uap10="http://schemas.microsoft.com/appx/manifest/uap/windows10/10"
  xmlns:rescap="http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities"
  IgnorableNamespaces="uap uap10 rescap">
  <Identity Name="{escape(name)}" Publisher="{escape(publisher)}" Version="{escape(version)}" ProcessorArchitecture="x64" />
  <Properties>
    <DisplayName>{escape(display)}</DisplayName>
    <PublisherDisplayName>{escape(publisher_display)}</PublisherDisplayName>
    <Description>{escape(desc)}</Description>
    <Logo>Assets\\StoreLogo.png</Logo>
  </Properties>
  <Resources>
    <Resource Language="en-us" />
  </Resources>
  <Dependencies>
    <TargetDeviceFamily Name="Windows.Desktop" MinVersion="10.0.19041.0" MaxVersionTested="10.0.22621.0" />
  </Dependencies>
  <Capabilities>
    <rescap:Capability Name="runFullTrust" />
    <Capability Name="internetClient" />
    <Capability Name="internetClientServer" />
    <Capability Name="privateNetworkClientServer" />
  </Capabilities>
  <Applications>
    <Application Id="App"
      Executable="{escape(exe)}"
      EntryPoint="Windows.FullTrustApplication"
      uap10:TrustLevel="mediumIL"
      uap10:RuntimeBehavior="packagedClassicApp">
      <uap:VisualElements
        DisplayName="{escape(display)}"
        Description="{escape(desc)}"
        BackgroundColor="transparent"
        Square150x150Logo="Assets\\Square150x150Logo.png"
        Square44x44Logo="Assets\\Square44x44Logo.png">
        <uap:DefaultTile Wide310x150Logo="Assets\\Wide310x150Logo.png" ShortName="{escape(display)}" />
        <uap:SplashScreen Image="Assets\\SplashScreen.png" BackgroundColor="transparent" />
      </uap:VisualElements>
    </Application>
  </Applications>
</Package>
"""
    path.write_text(xml, encoding="utf-8")


def content_types_xml(files: list[str]) -> bytes:
    defaults: dict[str, str] = {
        "xml": "application/xml",
        "png": "image/png",
        "exe": "application/vnd.microsoft.portable-executable",
        "dll": "application/x-msdownload",
        "json": "application/json",
        "txt": "text/plain",
        "env": "text/plain",
        "webp": "image/webp",
        "ttf": "application/octet-stream",
        "otf": "application/octet-stream",
        "wasm": "application/wasm",
        "dat": "application/octet-stream",
        "bin": "application/octet-stream",
        "pdb": "application/octet-stream",
    }
    overrides = {
        "/AppxManifest.xml": "application/vnd.ms-appx.manifest+xml",
        "/AppxBlockMap.xml": "application/vnd.ms-appx.blockmap+xml",
    }
    seen_ext: set[str] = set()
    lines = [
        '<?xml version="1.0" encoding="utf-8"?>',
        '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">',
    ]
    for rel in files:
        ext = Path(rel).suffix.lstrip(".").lower()
        if not ext or ext in seen_ext:
            continue
        seen_ext.add(ext)
        ctype = defaults.get(ext, "application/octet-stream")
        lines.append(f'  <Default Extension="{escape(ext)}" ContentType="{escape(ctype)}" />')
    for part, ctype in overrides.items():
        lines.append(f'  <Override PartName="{escape(part)}" ContentType="{escape(ctype)}" />')
    lines.append("</Types>")
    return ("\n".join(lines) + "\n").encode("utf-8")


def file_blocks(data: bytes) -> list[tuple[bytes, int]]:
    out: list[tuple[bytes, int]] = []
    if not data:
        digest = hashlib.sha256(b"").digest()
        out.append((digest, 0))
        return out
    for i in range(0, len(data), BLOCK):
        chunk = data[i : i + BLOCK]
        out.append((hashlib.sha256(chunk).digest(), len(chunk)))
    return out


def b64(digest: bytes) -> str:
    import base64

    return base64.b64encode(digest).decode("ascii")


def local_header_size(name_utf8: bytes, extra_len: int = 0) -> int:
    # ZIP local file header: 30 + filename + extra
    return 30 + len(name_utf8) + extra_len


def build_blockmap(entries: list[tuple[str, bytes]]) -> bytes:
    # Name uses backslashes in BlockMap (Windows packaging convention).
    lines = [
        '<?xml version="1.0" encoding="UTF-8" standalone="no"?>',
        '<BlockMap xmlns="http://schemas.microsoft.com/appx/2010/blockmap" '
        'HashMethod="http://www.w3.org/2001/04/xmlenc#sha256">',
    ]
    for name, data in entries:
        zip_name = name.replace("\\", "/")
        name_attr = zip_name.replace("/", "\\")
        name_bytes = zip_name.encode("utf-8")
        lfh = local_header_size(name_bytes)
        lines.append(
            f'  <File Name="{escape(name_attr)}" Size="{len(data)}" LfhSize="{lfh}">'
        )
        for digest, size in file_blocks(data):
            if size == 0 and len(data) == 0:
                lines.append(f'    <Block Hash="{b64(digest)}" />')
            else:
                lines.append(f'    <Block Hash="{b64(digest)}" Size="{size}" />')
        lines.append("  </File>")
    lines.append("</BlockMap>")
    return ("\n".join(lines) + "\n").encode("utf-8")


def should_skip(rel: Path) -> bool:
    if rel.name in SKIP_NAMES:
        return True
    if rel.suffix.lower() in {".wxs", ".msi", ".wixpdb"}:
        return True
    return False


def collect_payload(payload: Path, stage: Path, exe: str) -> None:
    exe_src = payload / exe
    if not exe_src.is_file():
        raise SystemExit(f"missing exe in payload: {exe_src}")

    for root, dirs, files in os.walk(payload):
        root_p = Path(root)
        # Skip MSI staging junk if present
        dirs[:] = [d for d in dirs if d not in {"msi-stage", ".git"}]
        for fn in files:
            src = root_p / fn
            rel = src.relative_to(payload)
            if should_skip(rel):
                continue
            dest = stage / rel
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(src, dest)


def pack_zip(out_msix: Path, files: list[tuple[str, bytes]]) -> None:
    """Write APPX zip with ZIP_STORED so BlockMap LfhSize matches local headers."""
    out_msix.parent.mkdir(parents=True, exist_ok=True)
    if out_msix.exists():
        out_msix.unlink()

    special: dict[str, bytes] = {}
    rest: list[tuple[str, bytes]] = []
    for name, data in files:
        key = name.replace("\\", "/")
        if key in ("[Content_Types].xml", "AppxBlockMap.xml"):
            special[key] = data
        else:
            rest.append((key, data))
    rest.sort(key=lambda x: x[0].lower())

    ordered: list[tuple[str, bytes]] = []
    for key in ("[Content_Types].xml", "AppxBlockMap.xml"):
        if key not in special:
            raise SystemExit(f"internal: missing {key}")
        ordered.append((key, special[key]))
    ordered.extend(rest)

    # Manual ZIP (no extra fields, no ZIP64) so LfhSize = 30 + len(name).
    central: list[bytes] = []
    with out_msix.open("wb") as fp:
        for name, data in ordered:
            name_b = name.encode("utf-8")
            if any(b > 127 for b in name_b):
                raise SystemExit(f"non-ASCII path not supported in MSIX packer: {name}")
            crc = zipfile.crc32(data) & 0xFFFFFFFF
            size = len(data)
            if size >= 0xFFFFFFFF:
                raise SystemExit(f"file too large for non-ZIP64 APPX: {name}")
            offset = fp.tell()
            # Local file header
            fp.write(struct.pack(
                "<IHHHHHIIIHH",
                0x04034B50,  # sig
                20,  # version needed
                0,  # flags
                0,  # compression = stored
                0,  # mod time
                0,  # mod date
                crc,
                size,
                size,
                len(name_b),
                0,  # extra len
            ))
            fp.write(name_b)
            fp.write(data)
            # Central directory header
            central.append(struct.pack(
                "<IHHHHHHIIIHHHHHII",
                0x02014B50,
                20,  # version made by
                20,  # version needed
                0,  # flags
                0,  # compression
                0,  # time
                0,  # date
                crc,
                size,
                size,
                len(name_b),
                0,  # extra
                0,  # comment
                0,  # disk start
                0,  # int attr
                0o100644 << 16,  # ext attr (regular file)
                offset,
            ) + name_b)

        cd_start = fp.tell()
        for chunk in central:
            fp.write(chunk)
        cd_size = fp.tell() - cd_start
        fp.write(struct.pack(
            "<IHHHHIIH",
            0x06054B50,
            0,  # disk
            0,  # cd disk
            len(central),
            len(central),
            cd_size,
            cd_start,
            0,  # comment len
        ))


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--payload", type=Path, required=True)
    ap.add_argument("--out", type=Path, required=True)
    ap.add_argument("--layout", type=Path, help="Also write unpacked layout here")
    ap.add_argument("--name", required=True, help="Identity Name (stable Store id)")
    ap.add_argument("--publisher", required=True, help='Publisher e.g. CN=Kumkaj')
    ap.add_argument("--publisher-display", default="Kumkaj")
    ap.add_argument("--display", required=True)
    ap.add_argument("--version", required=True)
    ap.add_argument("--exe", required=True)
    ap.add_argument("--description", default="")
    ap.add_argument("--icon-png", type=Path, default=None)
    args = ap.parse_args()

    version = four_part_version(args.version)
    payload = args.payload.resolve()
    if not payload.is_dir():
        raise SystemExit(f"payload not a directory: {payload}")

    with tempfile.TemporaryDirectory(prefix="kumkaj-msix-") as tmp:
        stage = Path(tmp) / "layout"
        stage.mkdir()
        collect_payload(payload, stage, args.exe)
        write_logos(stage / "Assets", args.icon_png)
        write_manifest(
            stage / "AppxManifest.xml",
            name=args.name,
            publisher=args.publisher,
            display=args.display,
            publisher_display=args.publisher_display,
            version=version,
            exe=args.exe,
            description=args.description,
        )

        # Read all staged files into memory for BlockMap (package is ~250MB — OK).
        file_entries: list[tuple[str, bytes]] = []
        for root, _, files in os.walk(stage):
            for fn in files:
                src = Path(root) / fn
                rel = src.relative_to(stage).as_posix()
                file_entries.append((rel, src.read_bytes()))
        file_entries.sort(key=lambda x: x[0].lower())

        ctypes = content_types_xml([n for n, _ in file_entries] + ["AppxBlockMap.xml"])
        # BlockMap covers Content_Types + payload; not itself or the signature.
        blockmap = build_blockmap([("[Content_Types].xml", ctypes)] + file_entries)

        all_files = [("[Content_Types].xml", ctypes), ("AppxBlockMap.xml", blockmap)] + file_entries
        pack_zip(args.out.resolve(), all_files)

        if args.layout:
            layout = args.layout.resolve()
            if layout.exists():
                shutil.rmtree(layout)
            shutil.copytree(stage, layout)
            (layout / "AppxBlockMap.xml").write_bytes(blockmap)
            # Content types lives at zip root; copy for MakeAppx users.
            (layout / "_Content_Types.xml").write_bytes(ctypes)

    size = args.out.stat().st_size
    print(f"MSIX → {args.out} ({size // (1024 * 1024)} MB)")
    print(f"Identity Name={args.name} Publisher={args.publisher} Version={version}")
    print("Unsigned. Sign with SignTool (Partner Center .pfx) before Store upload / sideload.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
