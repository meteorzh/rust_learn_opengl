#version 330 core
in vec3 position;
in vec2 texture;

out vec2 TexCoords;

void main()
{
    gl_Position = vec4(position, 1.0);
    TexCoords = texture;
}