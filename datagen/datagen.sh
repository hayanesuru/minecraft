#!/bin/bash
set -e
stat "./server.jar"
mkdir -p "./work/"
cd "./work/"
jar -xf "../server.jar"
java --class-path "$(find . -type f -name '*.jar' | tr '\n' ':')" "../Datagen.java"
mv -f ./*.txt "../"
cd ".."
rm -r "./work/"
