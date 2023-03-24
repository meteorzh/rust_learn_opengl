#version 330 core
in vec2 position;
in vec2 texture;

out vec2 TexCoords;

uniform mat4 model;
uniform mat4 projection;

void main()
{
    TexCoords = texture;
    gl_Position = projection * model * vec4(position, 0.0, 1.0);
}