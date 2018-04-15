attribute vec3 aPos;
attribute vec2 aTexCoord;

varying highp vec2 TexCoord;

void main()
{
    gl_Position = vec4(aPos, 1.0);
    TexCoord = aTexCoord;
}