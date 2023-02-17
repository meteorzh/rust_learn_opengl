#version 330 core
in vec3 position;
in vec2 texture;

out vec2 TexCoords;

void main()
{
    TexCoords = texture;
    gl_Position = vec4(position, 1.0);
}