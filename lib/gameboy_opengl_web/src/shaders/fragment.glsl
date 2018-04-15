varying highp vec2 TexCoord;

uniform sampler2D screen;

void main()
{
    gl_FragColor = texture2D(screen, TexCoord);
}