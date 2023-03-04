#version 330 core
out vec4 FragColor;

in vec3 WorldPos;
in int face;

uniform sampler2D equirectangularMap;

const vec2 invAtan = vec2(0.1591, 0.3183);
vec2 SampleSphericalMap(vec3 v)
{
    vec2 uv = vec2(atan(v.z, v.x), asin(v.y));
    uv *= invAtan;
    uv += 0.5;
    return uv;
}

void main()
{		
    vec2 uv = SampleSphericalMap(normalize(WorldPos));
    vec3 color = texture(equirectangularMap, uv).rgb;
    
    // FragColor = vec4(color, 1.0);
    if (face == 0)
    {
        FragColor = vec4(0.1, 0.0, 0.0, 1.0);
    }
    else if (face == 1)
    {
        FragColor = vec4(0.0, 0.1, 0.0, 1.0);
    }
    else if (face == 2)
    {
        FragColor = vec4(0.0, 0.0, 0.1, 1.0);
    }
    else if (face == 3)
    {
        FragColor = vec4(0.1, 0.2, 0.0, 1.0);
    }
    else if (face == 4)
    {
        FragColor = vec4(0.1, 0.0, 0.2, 1.0);
    }
    else if (face == 5)
    {
        FragColor = vec4(0.0, 0.2, 0.2, 1.0);
    }
}
