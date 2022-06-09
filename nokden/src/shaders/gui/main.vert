#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (location = 0) out vec4 o_color;

layout (push_constant) uniform Constants
{
    mat4 mvp;
    vec4 color;
    vec2[4] rect;
} pc;

void main()
{
    o_color = pc.color;
    gl_Position = pc.mvp * vec4(pc.rect[gl_VertexIndex], 0.0, 1.0);
}