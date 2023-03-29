#version 330 core
in vec2 position;
in vec2 texture;

out vec2 TexCoords;
out vec4 ParticleColor;

uniform mat4 projection;
uniform vec2 offset;
uniform vec4 color;

void main()
{
    float scale = 10.0f;
    TexCoords = texture;
    ParticleColor = color;
    gl_Position = projection * vec4((position * scale) + offset, 0.0, 1.0);
}