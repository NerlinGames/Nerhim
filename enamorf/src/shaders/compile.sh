#!/bin/bash

echo "Compile Vulkan shaders..."
COMPILER=/home/pin1776/Downloads/vulkansdk-linux-x86_64-1.3.211.0/1.3.211.0/x86_64/bin/glslc

$COMPILER mesh/main.vert -o mesh/main.spv_v
$COMPILER mesh/main.frag -o mesh/main.spv_f

echo "... done."