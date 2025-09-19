#!/bin/sh
cp ./target/release/{NAME} ./macOS/{NAME-UPPERCASE}.app/Contents/MacOS/{NAME}
ditto -c -k --sequesterRsrc --keepParent ./macOS/{NAME-UPPERCASE}.app ./releases/{NAME-UPPERCASE}-v{VERSION}app.zip