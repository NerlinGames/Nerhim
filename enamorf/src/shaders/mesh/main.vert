#version 400
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (location = 0) in vec3 pos;
layout (location = 1) in vec4 color;

layout (location = 0) out vec4 o_color;

layout (push_constant) uniform Constants
{
    mat4 camera;
    mat4 model_position;
} pc;

void main()
{
    o_color = color;
    gl_Position = pc.camera * pc.model_position * vec4(pos, 1.0);
}