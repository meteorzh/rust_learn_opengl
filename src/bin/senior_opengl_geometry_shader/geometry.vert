#version 330 core

in vec3 position;

void main()
{
    gl_Position = vec4(position.x, position.y, 0.0, 1.0); 
}