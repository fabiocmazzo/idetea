# El manifiesto de IDE TEA

<p><a href="MANIFESTO.md">English</a> · <a href="MANIFESTO.pt-BR.md">Português</a> · <b>Español</b></p>

El editor es donde pasas el día. Debería sentirse como una sala tranquila, no como un centro comercial.

## 1. La rapidez es una función

Esperar rompe la concentración. IDE TEA abre en un parpadeo, abre archivos grandes sin congelarse y hace el trabajo
lento (búsqueda, git, servidores de lenguaje) lejos de la pantalla donde escribes. Cuando algo va lento, es un bug.

## 2. Silencio por defecto

Sin cuenta. Sin telemetría. Sin ventanas emergentes sobre funciones que no pediste. La red solo se usa cuando tú lo
pides (push, pull, fetch). Nada cambia bajo tus pies: la configuración vive en un archivo TOML simple que puedes
leer, versionar y compartir.

## 3. Predecible antes que ingenioso

Todos los atajos están en un solo lugar (`F1`) y se pueden cambiar. Un panel se queda donde lo pusiste. Cerrar algo
con cambios sin guardar siempre pregunta. Borrar un archivo lo manda a la papelera. Los archivos cambiados fuera del
IDE se notan, y tu trabajo sin guardar nunca se tira para "ayudarte".

## 4. Pequeño por dentro, completo por fuera

El núcleo solo edita archivos. El árbol, la terminal, git, el depurador y todo lo demás son extensiones que puedes
desactivar. Un núcleo pequeño es más fácil de confiar, más fácil de leer y más difícil de romper.

## 5. Los agentes son bienvenidos, y tú sigues al mando

Los agentes de código son parte de cómo trabajamos hoy. IDE TEA les da una forma simple, abierta y hecha de archivos
de hablar contigo: preguntas como formularios de verdad, terminales que pueden pedir, ayudantes en su propio
worktree. Las reglas son siempre las mismas:

- **Lo ves antes de que se ejecute.** Cada comando que pide un agente aparece completo y espera tu aprobación, salvo
  que lo pongas en una lista en *tu* configuración. Un proyecto nunca puede darse permiso a sí mismo.
- **Tu respuesta es la respuesta.** Aprobar es una decisión, no un clic para quitarse un diálogo de encima. Los
  botones que importan esperan a que hayas hecho los pasos.
- **Archivos simples, sin ataduras.** El protocolo es JSON en una carpeta. Cualquier agente que lea y escriba
  archivos puede usarlo, y tú puedes leer cada mensaje.

## 6. Gratis de verdad

IDE TEA es Apache 2.0 y gratuito, y lo seguirá siendo. Ningún plan de pago escondiendo las partes buenas.

## 7. Hecho para mentes diferentes

El nombre tiene un segundo sentido: en portugués (y en español), TEA es la sigla del trastorno del espectro autista.
IDE TEA está hecho por un desarrollador autista, y se nota en lo que valora: poco ruido, texto claro, cosas que se
comportan siempre igual y espacio para concentrarse. Si tu mente funciona así también, bienvenido. Si no, es probable
que te guste igual.
