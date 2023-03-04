#version 330 core
out vec4 FragColor;

in vec3 FragPos;

uniform samplerCube image;

void main()
{
    FragColor = texture(image, FragPos);
    // FragColor = vec4(0.2, 0.0, 0.1, 1.0);
}