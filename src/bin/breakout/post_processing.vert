#version 330 core
in vec2 position;
in vec2 texture;

out vec2 TexCoords;

uniform bool chaos;
uniform bool confuse;
uniform bool shake;
uniform float time;

void main()
{
    gl_Position = vec4(position, 0.0f, 1.0f); 
    vec2 tex = texture;
    if(chaos)
    {
        float strength = 0.3;
        vec2 pos = vec2(tex.x + sin(time) * strength, tex.y + cos(time) * strength);        
        TexCoords = pos;
    }
    else if(confuse)
    {
        TexCoords = vec2(1.0 - tex.x, 1.0 - tex.y);
    }
    else
    {
        TexCoords = tex;
    }
    if (shake)
    {
        float strength = 0.01;
        gl_Position.x += cos(time * 10) * strength;        
        gl_Position.y += cos(time * 15) * strength;        
    }
}