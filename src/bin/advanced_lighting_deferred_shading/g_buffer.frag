#version 330 core
out vec3 GPosition;
out vec3 GNormal;
out vec4 GColorSpecular;

in vec2 TexCoords;
in vec3 FragPos;
in vec3 Normal;

uniform sampler2D texture_diffuse1;
uniform sampler2D texture_specular1;

void main()
{    
    // store the fragment position vector in the first gbuffer texture
    GPosition = FragPos;
    // also store the per-fragment normals into the gbuffer
    GNormal = normalize(Normal);
    // and the diffuse per-fragment color
    GColorSpecular.rgb = texture(texture_diffuse1, TexCoords).rgb;
    // store specular intensity in gAlbedoSpec's alpha component
    GColorSpecular.a = texture(texture_specular1, TexCoords).r;
}