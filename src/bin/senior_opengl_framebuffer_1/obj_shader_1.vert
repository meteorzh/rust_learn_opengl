#version 330 core
in vec3 position;
in vec2 texture;

out vec2 TexCoords;

void main()
{
    TexCoords = texture;    
    gl_Position = vec4(position.x, position.y, 0.0, 1.0);
}