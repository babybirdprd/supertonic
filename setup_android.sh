#!/bin/bash
set -e

# Setup script for Android support in Supertonic Tauri App

# 1. Download ONNX Runtime Android Libraries
# We need libonnxruntime.so for Android architectures.
# We will fetch them from Maven Central (onnxruntime-android aar).

ORT_VERSION="1.19.0" # Matches ort 2.0.0-rc.7 roughly (or we can use 1.18.0)
# Note: ort 2.0.0-rc.7 is compatible with 1.19.0.

echo "Downloading ONNX Runtime Android AAR (v$ORT_VERSION)..."
wget -q "https://repo1.maven.org/maven2/com/microsoft/onnxruntime/onnxruntime-android/$ORT_VERSION/onnxruntime-android-$ORT_VERSION.aar" -O onnxruntime.aar

echo "Extracting libraries..."
unzip -q -o onnxruntime.aar -d onnxruntime_extracted

# 2. Define target directory
# We place libs in a location where the Android Gradle project will pick them up.
# The `tauri android init` command creates `src-tauri/gen/android`.
# We assume the user has run `tauri android init` or will run it.
# If `src-tauri/gen/android` does not exist, we place them in a temporary folder
# and instruct the user to copy them or re-run this script after init.

ANDROID_DIR="examples/tauri-app/src-tauri/gen/android"
LIBS_DIR="$ANDROID_DIR/app/src/main/jniLibs"

if [ ! -d "$ANDROID_DIR" ]; then
    echo "Warning: Android project not found at $ANDROID_DIR."
    echo "Please run 'tauri android init' in 'examples/tauri-app' first."
    echo "Staging libraries in ./android_libs for now."
    LIBS_DIR="./android_libs"
fi

mkdir -p "$LIBS_DIR"

# Copy architectures
# AAR layout: jni/<arch>/libonnxruntime.so
# jniLibs layout: <arch>/libonnxruntime.so

ARCHS=("arm64-v8a" "armeabi-v7a" "x86" "x86_64")

for arch in "${ARCHS[@]}"; do
    if [ -d "onnxruntime_extracted/jni/$arch" ]; then
        echo "Installing $arch..."
        mkdir -p "$LIBS_DIR/$arch"
        cp "onnxruntime_extracted/jni/$arch/libonnxruntime.so" "$LIBS_DIR/$arch/"
    else
        echo "Warning: Architecture $arch not found in AAR."
    fi
done

echo "Cleaning up..."
rm onnxruntime.aar
rm -rf onnxruntime_extracted

echo "Success! ONNX Runtime libraries installed to $LIBS_DIR."
echo ""
echo "Next Steps:"
echo "1. If you haven't already, run 'tauri android init' in examples/tauri-app/src-tauri"
echo "   (If you just did that, verify the libs are in gen/android/app/src/main/jniLibs)"
echo "2. Ensure your Android device/emulator is connected."
echo "3. Run 'tauri android dev' to build and deploy."
