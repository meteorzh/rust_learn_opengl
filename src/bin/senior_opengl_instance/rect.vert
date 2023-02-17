#version 330 core
in vec2 position;
in vec3 color;

out vec3 fColor;

uniform vec2 offsets[100];

void main()
{
    vec2 offset = offsets[gl_InstanceID];
    gl_Position = vec4(position + offset, 0.0, 1.0);
    fColor = color;
}