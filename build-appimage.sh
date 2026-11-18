#!/usr/bin/env bash
cd "$(dirname "$0")/src-tauri" || exit 1
sudo -v || exit 1                       # şifreyi baştan bir kez sorsun
sudo systemctl stop vmware 2>/dev/null

restore() {
  for d in vmware vmware-installer; do
    [ -d /opt/_hide_$d ] && sudo mv /opt/_hide_$d /usr/lib/$d
  done
}
trap restore EXIT INT

for d in vmware vmware-installer; do
  [ -d /usr/lib/$d ] && sudo mv /usr/lib/$d /opt/_hide_$d
done

rm -rf ../target/release/bundle/appimage
NO_STRIP=true APPIMAGE_EXTRACT_AND_RUN=1 cargo tauri build --bundles appimage --verbose > /tmp/appimage.log 2>&1
echo "çıkış kodu: $?"
grep -nE "ERROR|Could not" /tmp/appimage.log | tail -10
ls -la ../target/release/bundle/appimage/*.AppImage 2>/dev/null
