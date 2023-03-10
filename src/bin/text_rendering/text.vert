#version 330 core
in vec2 position;
in vec2 texture;

out vec2 TexCoords;

uniform mat4 projection;

void main()
{
    gl_Position = projection * vec4(position, 0.0, 1.0);
    TexCoords = texture;
}