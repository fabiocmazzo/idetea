## Preguntas por el IDE (panel Interfaces)

Cuando necesites que elija entre opciones, confirme algo o complete unos pocos campos, puedes crear una interfaz
en el IDE TEA en lugar de preguntar en el chat. Este texto basta; el formato completo está en
https://github.com/fabiocmazzo/idetea/blob/main/docs/interfaces/formato.md

1. Escribe `.idetea/ui/<nombre>.json` (nombre: minúsculas, números, `_` y `-`). Escribe en un archivo temporal y
   renómbralo, para que el IDE nunca vea el archivo a medias: `cat > .idetea/ui/.<nombre>.json.tmp` y después
   `mv` al nombre final.
   ```json
   {"format": "idetea-ui", "version": 1, "revision": 1, "title": "…", "source": "<tu nombre>",
    "data": {"opcion": "a", "pruebas": true},
    "components": [
      {"id": "q", "component": "text", "text": "…"},
      {"id": "op", "component": "select", "label": "…", "value": "/opcion", "options": ["a", {"value": "b", "label": "B"}]},
      {"id": "t", "component": "checkbox", "label": "Escribir pruebas", "value": "/pruebas"},
      {"id": "n", "component": "input", "label": "Nombre", "placeholder": "opcional", "value": "/nombre"},
      {"id": "ok", "component": "button", "label": "Confirmar", "action": {"name": "ok", "context": ["/opcion", "/pruebas", "/nombre"]}},
      {"id": "no", "component": "button", "label": "Cancelar", "style": "secondary", "action": {"name": "cancelar"}}
    ]}
   ```
   - Componentes: `text` (`style`: `muted`, `warning`, `error`), `select`, `checkbox` (booleano), `input`
     (texto) y `button` (`style`: `secondary`, `danger`). `value` es un puntero JSON a un valor en `data`;
     `data` trae los valores iniciales.
   - En la respuesta solo vienen las rutas listadas en el `context` del botón pulsado; un campo nunca completado
     viene como `null`.
   - `action.name` es un texto libre que vuelve en la respuesta para que sepas qué botón se pulsó.
   - Después de un clic, los botones quedan desactivados hasta que escribas una revisión nueva.
2. Espera la respuesta en `.idetea/ui/responses/<nombre>/`. Usa bash (en zsh, `compgen` no existe o se comporta
   distinto):
   ```sh
   timeout 600 bash -c 'until ls .idetea/ui/responses/<nombre>/*.json >/dev/null 2>&1; do sleep 1; done'
   ```
   Cada archivo es una respuesta:
   `{"action": "ok", "values": {"/opcion": "b", "/pruebas": true, "/nombre": null}, "revision": 1, "seq": 1, ...}`.
   `action` es el `action.name` del botón; `revision` es la revisión que yo veía al pulsar; `seq` es el orden
   (puede haber más de un archivo; léelos en orden de nombre). Léelo y **borra el archivo de la respuesta**: así
   el IDE sabe que lo recibiste.
3. Para mostrar un resultado o hacer otra pregunta, reescribe el mismo archivo con una `revision` mayor. Un
   documento solo con `text`, sin botones, sirve para mostrar algo sin esperar respuesta.
4. Al terminar, borra `.idetea/ui/<nombre>.json`. El resto de la carpeta (`.gitignore`, `responses/`) es del
   IDE; déjalo.

No pongas secretos en el documento. Si no llega respuesta en 10 minutos (el panel puede estar desactivado),
pregunta en el chat.
