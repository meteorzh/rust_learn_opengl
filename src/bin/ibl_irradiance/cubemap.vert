#version 330 core
in vec3 position;

// out vec3 WorldPos;

void main()
{
    // WorldPos = position;
    gl_Position =  vec4(position, 1.0);
}