#!/bin/sh
rm -rf /tmp/app
mkdir /tmp/app
./linux/squashfs-root/AppRun --appdir /tmp/app -d ./linux/{NAME-UPPERCASE}.desktop -i ./linux/{NAME}.png -e ./target/release/{NAME} --output appimage
mv {NAME-UPPERCASE}-*.AppImage ./releases/{NAME-UPPERCASE}-v{VERSION}.AppImage