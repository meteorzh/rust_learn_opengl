#version 330 core
in vec2 position;
in vec3 color;
in vec2 offset;

out vec3 fColor;

void main()
{
    vec2 pos = position * (gl_InstanceID / 100.0);
    gl_Position = vec4(pos + offset, 0.0, 1.0);
    fColor = color;
}