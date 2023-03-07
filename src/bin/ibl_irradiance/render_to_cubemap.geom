#version 330 core
layout (triangles) in;
layout (triangle_strip, max_vertices=18) out;

uniform mat4 cube_views[6];
uniform mat4 capture_projection;

out vec3 WorldPos;

void main()
{
    for(int face = 0; face < 6; ++face)
    {
        gl_Layer = face; // built-in variable that specifies to which face we render.
        for(int i = 0; i < 3; ++i) // for each triangle vertex
        {
            WorldPos = gl_in[i].gl_Position.xyz;
            gl_Position = capture_projection * cube_views[face] * gl_in[i].gl_Position;
            EmitVertex();
        }
        EndPrimitive();
    }
}