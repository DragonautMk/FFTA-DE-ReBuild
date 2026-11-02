<a id="ffta-estado-de-investigacion-del-fallo-real"></a>

# FFTA: investigation status for the reported issue

2026-10-18: USA Pub separated into the right corner; USA shop uses a centered
canvas; the USA empty-bank notice was identified. Shared USA service windows
are recognized by owner; the actual Prison scene remains NOT MEASURED.
[ffta_usa_town_menus.md](ffta_usa_town_menus.md).

2026-10-17: mouse profile for Press Start and the USA main menu implemented
and tested by replay. [ffta_usa_frontend_mouse.md](ffta_usa_frontend_mouse.md).

2026-10-17: user confirms magic effects already corrected; do not touch them.
Lightning damage banner measured in WIN0 [16,8,224,24], extracted in full
without the X=160 clipping. [ffta_spell_help_banner.md](ffta_spell_help_banner.md).

2026-10-17: Lester's action menu is split at X=160; its actual frame begins
at X=152. Extraction follows the BG2 header/footer, pending feedback.
[ffta_action_frame_width.md](ffta_action_frame_width.md).
Next pending issue: Thunder/Pyro magic effects repeated on both sides.

2026-10-17: Clara's ear tip in Giza captured live and corrected.
[ffta_portrait_ears.md](ffta_portrait_ears.md): two pieces at Y=48/56 excluded
by the old Y>=64 filter now share the portrait layer.

2026-10-17: Comprar/Vender centered; compact Vender categories and Monster Bank
measured by replay with coordinate clicks. Details and limits:
[ffta_shop_bank_mouse.md](ffta_shop_bank_mouse.md).

Priority: reproduce ffta-batalla-3.input before claiming a solution.
The earlier 44-object test does not reproduce the issue reported by the user.
No further game logic change is applied without approval.
No commits or installer changes in this investigation.

MEASURED: AGENTS.md read and git status --short run; the repository already
contains staged, unstaged and untracked changes.
MEASURED: initial release build passed in 0.42 s; diagnostics build passed in
26.86 s; recomp tests: 6 passed.

MEASURED: ffta-batalla-3.input ends with an event at frame 12440.
It contains release and press events on the same frame, for example 11730.
InputScript sorted tuples (frame, mask), changing their recorded order.
The reader was corrected to sort stably by frame only.
This affects host replay, not the game logic or bytes.

MEASURED: runc executes the native path with the same Machine, MemMap, run_dma
and PPU as play, without a window, desktop audio, vsync or frame wait.
--sav was added to preload a copy into memory; runc does not write the .sav.
Input is replayed from frame 0, not from the start of the analyzed segment.

MEASURED: created E:/AI/ARC/ffta-respaldo.sav without modifying the original.
The copy, the original and ffta-batalla-3.input.sav have SHA256:
71189F7FB6AED638640078FBA3A35FDA6C39C8962E74DCC75935AAC948DA9063.
The replay uses another private copy: target/ffta-replay3/start.sav.
ASSUMPTION: HLE startup, based on the BIOS paths searched locally;
the user was asked whether the recording used a real BIOS.

Exact command for the first complete replay:

```powershell
$env:FFTA_WIDESCREEN='32:9'
$env:FFTA_PATCH_BG='1'
$env:FFTA_PATCH_SPR='1'
$env:FFTA_BG_EDGE='transparent'
$env:FFTA_WIDE_DEBUG=''
$env:FFTA_OAM_LOG='1'
$env:FFTA_OAM_LOG_FROM='10500'
$env:FFTA_OAM_CAPTURE_PREFIX='target/ffta-replay3/run1'
$env:FFTA_BENCH_LIBRARY='target/ffta-movement/native-normal.dll'
.\target\release\recomp.exe runc 'Final Fantasy Tactics Advance (USA).gba' --frames 12540 --input ffta-batalla-3.input --sav target/ffta-replay3/start.sav --out target/ffta-replay3/run1-final.ppm *> target/ffta-replay3/run1.log
```

The named DLL is an already compiled local copy of the user's ROM;
it must not be distributed. From frame 10500 the log records, per frame,
the BG camera, calls/arguments to the three known producers, received
descriptors, host list and OAM entries. Every 30 frames it records hashes of
both framebuffers, OAM and OBJ VRAM; every 120 it can save an image.
MEASURED: the replay reached battle; images of the final segment were generated.
Two complete runs from frame 0 produced 69 identical samples of the central
framebuffer, wide framebuffer, OAM and OBJ VRAM, frames 10500..12540.
Literal script output: determinism_samples 69 mismatches [].
Final FNV hash for both: abc92001e317eae0.
Final central SHA256 hash for both:
3ffb91edfe2e1bc047624f0b43c0c30b39770af3ba031d364a260f6b48f66bc2.
NOT MEASURED: user confirmation that the recorded startup was HLE;
the recording stores no BIOS metadata and no installed BIOS was found.

MEASURED: nine graphics instances are identified by the priority pointer
passed to the producer minus 0x1e, not by palette or r5.
In the final segment, calls for some instances disappear before
030065A8; there are no mismatches in the host list or transferred OAM.
For example, 0201af18 stops arriving after frame 12042, near the original
left edge. Measurement of its jumps and tiles remains unfinished.

MEASURED by reading: 030062EC, Thumb, ROM source 08A38EA0..08A38FC8,
calculates list visibility with limits 0..239 in X and 0..159 in Y.
08394028 contains its Thumb pointer 030062ED. Caller 0808F804 invokes it
at 0808F832 through thunk 08142254 and marks it visible only if it returns 1.
08021DA8 returns NULL if flags & 0x40 == 0; caller 0808F1F4 exits
at 0808F216 on NULL, before emitting objects.
MEASURED: the 296 RAM bytes match the ROM source, hash:
1e988998a21067596046e2b303da1ad9b6924c18146f396ce0bf5f6e038433c7.
MEASURED: at producer frame 12041, X=-7, the function returns 1 and
030065A8 is invoked for 0201af18. At frame 12042, X=-10, its six components
are rejected on the left; it returns 0 and the producer is no longer invoked.
The trace reads actual interpreter instructions on the native path:

The literal game disassembly was removed from the public source distribution.
Only independently described behavior, hook addresses and verification hashes are retained.

At 0300638A, the r0 values for the six right edges are
-2, -10, -2, -10, -2, -10. All are below 0 but within the
32:9 margin, whose left limit is -164.
Caller 0808F804 does not mark the visibility result when it returns 0.
MEASURED: it is lost before the producer, not through host publication failure.

MEASURED: the 320 OBJ VRAM bytes referenced by that instance in its last
received list remain identical in image frames 12041..12044:
8dac051f7fa7a16c7abd58bbaef877779ae968e1b88dbb2defc903151d6d84eb.
170 nonzero bytes, zero references outside OBJ VRAM. Therefore, freeing these
tiles does not explain this particular disappearance.
NOT MEASURED: all assets of units never visible or all maps.
Filtering at 03006170 adjusts priority for intersections; it is not a direct
comparison with 240, and no change is proposed based on similarity.

MEASURED: gamedb.sqlite returned no functions for this ROM in 0808E000..08090000;
these addresses came from actual calls and instruction reads.
Complete disassemblies and logs remain in target/ffta-replay3, ignored
by git. No ROM bytes or generated DLLs are included in the distribution.

MEASURED by reading: another Thumb producer exists at 030068A4, originating
at 08A39458, with comparisons against 239/159; it is not in the current hook.
MEASURED: observation of 030068A4 recorded no entries in frames
10500..12539. The DLL table has other=false for 030068A5: its entry
would have passed through the observed interpreter. It does not participate
in this segment; NOT MEASURED: its use in other scenes or animations.
MEASURED, frames 11334..12539: 080014C8 from LR 08001D85, 1377 calls;
080016D4 from LR 08001DF3, 1206; both with limit 48 and inside the hook.
030065A8 from LR 0808F313, 5645 calls, and LR 0808F6CD, 1204.
Other documented callers of 14C8/16D4 were not observed in that segment;
the two functions are hooked by PC regardless of caller.

MEASURED: exact captures proof-12042.png and proof-12044.png show the
character disappearing at the left margin while the background remains.
The producer period is labeled with bus.frames before execution;
the capture is labeled when the frame completes. Publication and the PPU
introduce the visible offset between rejection at 12042 and image 12044.
NOT MEASURED: an exclusive correspondence between each component and a logical
game unit; the identifiers are pointers to graphics instances.

MEASURED: final normal tests: 80 + 7 + 3 passed, 0 failed;
3 tests ignored by default. The new input-reader test passed.
Instruction and tile diagnostics retained the same final hash in additional
replays. The ROM, original .sav and game condition were not modified.
Applying a visibility patch is pending.

Previous state, superseded by the eight-unit measurement described below:
Minimal proposal to hook predicate 030062EC by PC,
active under FFTA_PATCH_SPR and only in widescreen. Preserve original
execution and its cycles; on return, use the result calculated for the
rectangle X=-extra..239+extra, keeping Y=0..159. Store input arguments
on the host for that calculation and check the ROM SHA256 and the
complete predicate contents in ROM/RAM before enabling it.
Extra=0, incorrect hash or different bytes: original result unchanged.
This changes the result seen by the game and requires approval before
implementation. The existing producer and i32 list would continue to avoid
the later rejection and the 9-bit X limit for expanded objects.
Proposed tests: both complete A/B replays, presence of the instance
12041..12046, tiles, original OAM, equivalence for 600 frames with extra=0 and
32:9/1536; also check producer 030068A4 if actual calls occur.

NOT MEASURED: no usable complete checkpoint was found to restore
CPU, RAM and peripherals. The recompiler's IWRAM snapshots are code images
and do not replace accumulated state. No checkpoint was implemented.

Update: per-unit tutorial metric
MEASURED: X hook implemented under FFTA_PATCH_VIS and FFTA_PATCH_SPR,
checking SHA256 and the ROM/RAM function. Original result with extra=0.
MEASURED: final 1201 frames: current fails in 995; proposed fails in 493.
UNRESOLVED: eight units are not published in every frame.
MEASURED: six final bodies remain in RAM but receive Y rejection before
the producer. There is also an additional Leslaie body, reported separately.
ASSUMPTION: the reference eight correspond to IDs 0..7, excluding that body.
MEASURED: extra=0 preserves all 1201 metrics and 41 hash samples.
Details, limits and complete lists in docs/ffta_character_metric.md.
The additional proposal to remove Y rejection has not been applied.

Siguiente investigacion: casillas durante zoom-out
MEDIDO POR EL USUARIO: personajes validados jugando en 32:9, ya no
desaparecen. El parche de personajes se conserva sin modificaciones.
Los fallos anteriores de ocho publicados no anulan esa validacion visual;
la definicion del octavo personaje y la diferencia con visibilidad Y
siguen documentadas en ffta_character_metric.md.
MEDIDO: ffta-batalla-4.input y su sidecar no estan en la raiz del repo.
Busqueda rg --files bajo E:/AI/ARC, filtrada por batalla-4, sin resultados.
NO MEDIDO: replay 4, determinismo, lista de rango en RAM, frames del fallo,
condicion del escritor BG0/BG1 y capturas antes/despues. Falta la grabacion.
MEDIDO por lectura: InputScript v1 contiene frame y mascara KEYINPUT;
el grabador escribe solo cambios de keys. No graba zoom_level ni extra_y.
El zoom se procesa aparte en play.rs por Minus/Equal. El replay headless
configure_headless_widescreen solo fija extra X; no reproduce zoom-out.
MEDIDO por lectura: MAX_ZOOM=2, MAX_EXTRA_Y=320; play limita los pasos por
MAX_ZOOM y por ancho/alto maximos. No equivale a poder alcanzar extra_y=320
desde 32:9 con la configuracion actual. El maximo real de la captura falta.
Logging anadido bajo FFTA_OAM_LOG: FFTA_VIEW al cambiar dimensiones,
con frame, nivel de zoom, extra X/Y y dimensiones de vista/render.
No cambia el estado emulado ni el zoom, no toca sprites ni instalador.
No se aplica un parche BG nuevo sin medir antes la causa.

Actualizacion zoom y casillas, prueba manual pendiente
MEDIDO: runner runc admite FFTA_ZOOM_LEVEL=n o calendario frame:nivel.
32:9 niveles 0..6: extra X 164,199,238,282,331,387,448;
extra Y 0,10,21,34,47,63,80. Variable ausente conserva el runner anterior.
MEDIDO: replays completos de batalla-3 coinciden en 1201 hashes finales.
Se construyo movement-freeze.input conservando todos los eventos y anadiendo
12540 3ff. No se requiere otra grabacion del usuario.
MEDIDO: matriz calculada en 02005CB0, 64x64; BG0 copia una ventana 32x32.
En la muestra final, 108 tiles RAM frente a 102 VRAM; 1024/1024 palabras
VRAM coinciden con el recorte RAM de origen (33,35).
MEDIDO: escritor Thumb 0801AF08, llamador 0801B80C. Productor de casillas
0801D130: 20 llamadas style=1, caller 0801D803, coordenadas logicas X/Y
y proyeccion en tiles, cuatro cuadrantes por casilla. La tabla de flags
02008190 contiene marcas parciales de vecinos; contar bytes no cuenta casillas.
MEDIDO: PPU solo escribia 160 filas. En zoom 6 las 160 filas adicionales
quedaban negras; capturas before-11347.png y frozen-13101.ppm.
IMPLEMENTADO PARA PRUEBA: FFTA_PATCH_MOVEMENT=1, requiere PATCH_BG y ROM
exacta, SHA256 de 0801AF08..0801B5D0 y coincidencia de los 1024 valores
de cada plano VRAM con su recorte RAM. Si no coincide usa la ruta anterior.
El host lee los planos RAM completos y termina las filas extra en batalla.
No cambia escrituras originales del juego, sus ciclos, el enganche de
personajes ni la politica de borde del terreno. Por defecto esta desactivado.
MEDIDO: extra=0, 600 frames, MOVEMENT=0/1: framebuffer y OAM identicos.
NO MEDIDO TODAVIA: exito visual del parche nuevo, metrica por casilla antes
y despues, y regresion de los 1201 frames con el parche nuevo activo.
Prueba manual preparada en target/ffta-zoom/manual/FFTA-movement-test.gba;
su save privado procede de ffta-respaldo.sav. No se toco ningun save original.
Actualizacion: el usuario confirma que las casillas ya no desaparecen.
MOVEMENT esta activo por defecto, con las mismas verificaciones de ROM y RAM.
MEDIDO: el escenario conserva planos RAM 64x64 en 0200B1A0 y 020091A0;
VRAM contiene recortes 64x32. El parche STAGE lee esos planos completos solo
tras verificar el recorte entero (2048 palabras por capa). No modifica el juego.
MEDIDO: comparacion STAGE=0/1: 1861 frames de personajes sin cambios,
incluidos los 1201 finales de la grabacion; 63 muestras de framebuffer central
y OAM sin cambios. Frame final: 8 personajes, sin grupos duplicados.
MEDIDO: extra=0, comparacion por frame durante 600 frames: framebuffer y OAM
identicos. Capturas stage-off-13101.png y stage-on-13101.png muestran mas tejado
y arquitectura lateral. La confirmacion en juego real sigue NO MEDIDO.
MEDIDO: las filas adicionales de zoom ahora se dibujan, no quedan vacias.
Input contiene Keyboard y Gamepad, remapeo y layout Xbox por defecto.
NO MEDIDO: mando real; no se prueba por instruccion del usuario.
Instalador Linux: codigo, script y workflow preparados; ejecucion NO MEDIDO.
La carpeta assets tiene referencias de compilacion y no se elimina.

Correccion del cielo durante zoom:
MEDIDO: replay frame 13100, palette0 cambia por linea: 7D21,7DA4,7E67,
7F2A,7FCC en lineas 0,40,80,120,159. El gradiente es raster de paleta,
no una capa affine o bitmap como se afirmo antes sin medir.
Las filas host adicionales leian la paleta de VBlank (azul inicial).
Ahora conservan los colores de los extremos antes de VBlank y prolongan
esos colores fuera de las 160 lineas. No se copian filas del escenario.
Actualizacion: prolongar un color fijo seguia dejando una banda lisa.
El host ahora extrapola por canal RGB555 el gradiente observado entre
las lineas 0 y 159; los pixeles originales de 240x160 no cambian.
MEDIDO: en frame 13101, filas 0/79/80/239/240/319 cambian RGB
de 0,0,255 a 8,74,255 a 99,247,255 a 140,255,255 sin salto en
los bordes de la pantalla original. Captura sky-extended-13101.png.
MEDIDO: 600 frames extra=0 framebuffer y OAM iguales; 1201 frames de
personajes del tutorial sin cambios respecto al parche validado.
Retiradas heuristicas OBJ VRAM/paleta/OAM no vacias: no resolvian las
siluetas segun el usuario y anadian exploraciones de memoria por linea.
Se intento una comprobacion de la paleta propia de cada pieza OBJ en
widescreen: evita dibujar una pieza cuando su banco de 16 colores esta
completamente vacio. No se ha confirmado si elimina las siluetas reales.
NO MEDIDO: causa de las siluetas transitorias; no aparecen en las muestras
10020,10200,10400,10600 del replay existente. No se dan por arregladas.

Zoom de escenario durante menus y cinematicas (2026-10-12):
MEDIDO EN CODIGO: la ruta de planos completos solo aceptaba BG2/BG3 y
exigia ambos fondos activos con scroll identico. El detector anterior tambien
admitia terreno en BG0/BG1, pero esas capas no leian el plano RAM completo.
CORRECCION: identificar cada plano por BGCNT 4C03/4E02 y su estado RAM
ya verificado, sin depender de la activacion ni del scroll de su pareja.
Se conserva la verificacion de ROM, FFTA_PATCH_STAGE y FFTA_PATCH_BG.
No se modifican datos del juego ni el parche de personajes.
MEDIDO: tests gba-core: 85 passed, 3 ignored. Prueba sintetica de pareja
oculta, cambio BG0/BG2, extension vertical y 600 frames con extra=0 iguales.
NO MEDIDO: replay del fallo real, hashes de framebuffer/OAM de 600 frames
reales y metrica de 8 personajes. No hay grabaciones .input en el repo
ni en la raiz E:/AI/ARC. Requiere validacion jugando; no se afirma resuelto.

Panel de stats y zoom (2026-10-12):
MEDIDO EN CODIGO: las filas extra excluian capas UI, pero dibujaban OBJ.
Cambio: incluir todas las capas texto activas tambien en las filas extra;
las UI no repiten verticalmente fuera de su mapa de tiles. Ruta extra=0 intacta.
MEDIDO: 86 tests passed, 3 ignored; prueba de panel en y=180 y no repeticion.
NO MEDIDO: reproduccion visual del panel del enemigo de la captura del usuario.

Panel al pasar el cursor sobre unidades (2026-10-12):
MEDIDO: replay derivado zoom-no-range.input recuperado del repo anterior.
La grabacion ffta-batalla-3.input de ese repo contiene solo su cabecera.
Dos ejecuciones desde frame 0 hasta 13000: hash f158877f466426fb igual.
Se usa exclusivamente target/ffta-stats/repro.sav, copia del respaldo.
MEDIDO: el panel de Ritz no es un BG: productor 080014C8, lista 083916FA,
cuatro piezas semitransparentes, tiles 1F0/200/208, paleta OBJ 11.
Todas las piezas y tiles existen. BLDCNT=3F43, BLDALPHA=090E: el fondo
se mezcla con terreno y parece parcialmente ausente; no es descarte de piezas.
Cambio de presentacion: FFTA_PATCH_STATS=1 por defecto vuelve opaco solo
ese fondo identificado en la lista host de la ROM exacta y funciones verificadas.
No cambia atributos OAM ni logica del juego; extra=0 conserva la ruta original.
MEDIDO: 87 tests normales y 3 con ROM local pasan. Frame 13000 cambia
2558 pixels, todos dentro de x600..687/y184..231; exterior identico y OAM igual.
MEDIDO: framebuffer y OAM iguales en 600 frames con extra=0, STATS=0/1.
Capturas hover-13000.png y after-13000.png en target/ffta-stats.
NO MEDIDO: panel rojo de enemigos; el replay construido no llego a mostrarlo.
La metrica de este recorrido derivado tiene frames con menos de 8: no se
considera aprobada la prueba de 8 constantes ni se toca el parche de personajes.
MEDIDO: comparacion STATS=0/1 en 1201 frames (11800..13000): metrica
identica en todos. 297 frames dan menos de 8 en ambas versiones de este
replay derivado con la biblioteca nativa recuperada; no es una regresion nueva.
La lista completa queda en target/ffta-stats/character-failures.json.
Windows Installer/payload y TAR Linux compilados y actualizados, sin commits.

Correccion del intento anterior de stats (2026-10-12):
El usuario confirma que quitar semitransparencia NO arreglo el recorte.
Se RESTAURA siempre el modo semi original; se descarta el intento opaco.
MEDIDO en VRAM: las cuatro piezas del fondo contienen una curva con
pixels transparentes: cobertura de 67 pixels en fila 104 y 88 en fila 127.
Esto es distinto de las capas BG del menu de acciones.
Cambio host: completar solo los texels transparentes de esas piezas
identificadas, usando el indice de color vivo del tile rectangular 208.
Se preservan paleta, letras, atributos OAM, prioridad y alpha original.
FFTA_PATCH_STATS=0 conserva forma original; 1 completa el fondo en widescreen.
No es una recuperacion de sprites o tiles ausentes: es completar la geometria
del fondo del panel, limitado a la plantilla verificada y sin tocar la ROM.
MEDIDO: 88 tests passed, 3 ignored. Test de huecos, color vivo, semitransparencia
y extra=0. Nueva comparacion real de 600 frames: framebuffer/OAM iguales.
NO MEDIDO: panel rojo; la validacion visual del usuario sigue pendiente.
MEDIDO: replay completo hasta 13000 con el cambio nuevo. Fondo sin huecos
y alpha preservado: cap-after-13000.png. Se completan 217 pixels, todos
dentro del panel; fuera del panel identico y atributos OAM identicos.
MEDIDO: metrica de personajes identica en los 1201 frames de STATS=0/1.
Los 297 frames con menos de 8 del recorrido derivado permanecen iguales.
El comando PowerShell devolvio 1 pese a llegar al frame 13000 y generar
los volcados; el log registra NativeCommandError para mensajes de stderr.
Los verificadores posteriores terminan con codigo 0; no se oculta ese retorno.
Windows y Linux compilados y actualizados, sin tocar saves originales.

Objetivo correcto del panel de stats (aclaracion del usuario):
El semicirculo NO estaba mal. El fallo es que el panel no llega al borde.
Se REVIERTE completar sus texels transparentes. Forma y alpha originales.
Nuevo STATS=1: solo las piezas rectangulares del tile 208 se prolongan
hasta el borde del framebuffer ancho. H-flip elige izquierda o derecha.
Se repite su columna exterior viva, con su textura, paleta y semitransparencia.
Capas, posiciones de texto, curva, OAM y logica de personajes intactas.
MEDIDO: 88 tests passed, 3 ignored. Test de ambas orientaciones, curva
sin rellenar, alpha y no extension con STATS=0 o extra=0.
MEDIDO: comparacion real de 600 frames extra=0: framebuffer/OAM iguales.
MEDIDO: reconstruccion del volcado privado en edge-preview.png muestra
la banda transparente hasta el borde derecho, con la curva conservada.
NO MEDIDO: replay completo nuevo de 1201 frames, panel rojo y validacion
jugando del usuario. El preview reconstruye el volcado, no es una captura nueva.
Windows y Linux compilados; Installer/payload y TAR actualizados, sin commits.

Menu de pausa nativo adaptado del HTML del usuario (2026-10-13):
Estilo pergamino, gradientes crema, bordes laton, esquinas ornamentales,
seleccion cuero/dorado y dibujos propios para iconos. Sin motor HTML,
red, dependencias nuevas ni fuentes o iconos web empaquetados.
Se conserva la fuente pixel nativa: Cinzel/Inter y los efectos Web Audio
no se trasladan literalmente. Valores reales de av.cfg/input.cfg conservados.
Input abre pestanas Keyboard/Gamepad y conserva el dispositivo elegido.
Listas desplazables por teclado/mando/rueda; boton Back siempre visible.
Selectores video desplegables con aceptar/cancelar; FF segmentado clickable.
Exit pide confirmacion con No seleccionado por defecto.
MEDIDO: 42 tests passed: input-config 12, menu 19, recomp 11.
Pruebas de dropdown, targets de remapeo con scroll, tabs, FF y salida segura.
MEDIDO: previews nativos generados/revisados para main, gamepad, FF,
salida y selector. Windows/Linux release compilados y payload/TAR actualizados.
NO MEDIDO: interaccion jugando con ventana/mando fisico y efectos del GPU
sobre el estilo; no se inicio una partida ni se tocaron saves originales.
Referencia HTML en target/pause-menu/reference.html; previews en esa carpeta.

## 2026-10-13 - Menu moderno e iconos de mando completos

Menu renderizado en un canvas independiente de la GBA, sin alterar el framebuffer
del juego. Cinzel/Inter con antialiasing y avisos OFL en Licenses/fonts.
Iconos vectoriales para A/B/X/Y, LB/RB, LT/RT, View/Menu/Guide, L3/R3,
las cuatro direcciones del D-pad y las direcciones de ambos sticks.
Ratones usan coordenadas del canvas; GPU y fallback CPU presentan el mismo menu.
Asignaciones y parches del juego conservados. Preview: target/pause-menu/gamepad-modern.png.
Prueba fisica del mando: NO MEDIDO.

## 2026-10-16 - TinyCC en memoria para Windows y Linux
MEDIDO: mismo TinyCC que Android, fuente 43c7708b85681a2fd4451c8a541af4494a8919b2.
MEDIDO: EUR build completo 3.611 s; USA 4.706 s; USA GCC anterior 116.634 s.
MEDIDO: cache EUR 0.079 s; framebuffer/OAM iguales a GCC en 600 frames de ambas ROMs.
MEDIDO: USA extra=0 identico con parches off/on en 600 frames.
MEDIDO: 141 tests existentes y 2 pruebas TinyCC adicionales pasaron.
MEDIDO: replay 12540 frames igual a GCC; metrica antigua cuenta 4 en ambos, no valida 8.
MEDIDO: Windows y Linux compilados; no se empaqueta GCC. TinyCC incluye su fuente exacto y licencia.
NO MEDIDO: ejecucion Linux, no hay WSL instalado en este host.
Detalles y limites en docs/desktop_tinycc.md. Sin cambios en sprites, fondos o gamepad.

## 2026-10-16 - Dialogos anclados al borde y tamano independiente del zoom
MEDIDO: snapshot EUR separa BG2 texto, BG3 fondo y cinco piezas OAM 8-bpp de retrato.
MEDIDO: previews 32:9 x1/x2 muestran texto, fondo y retrato juntos al borde, sin extension.
MEDIDO: 99 tests core pasaron; 600 frames extra=0 con layout off/on dan framebuffer/OAM identicos.
Windows compilado en recomp-dialogue-layout.exe y copiado al recomp.exe habitual y payload.
NO MEDIDO: validacion jugando; no se afirma validacion visual USA ni otros formatos de dialogo.
Punto futuro de Settings: MemMap.ffta_dialogue.style (scale, offset_x, offset_y), sin menu nuevo.
A/B: FFTA_DIALOGUE_LAYOUT=0/1, por defecto 1. Detalles en docs/dialogue_layout.md.


2026-10-16: Text Bubbles submenu under Video: persistent size and backing length.
Exact desktop zoom/crop now drives uniform dialogue scale. Host-list objects cannot
be treated as portrait pieces. Parked sentinel filter also covers horizontal-only wide.
MEDIDO: 157 tests passed; EUR extra=0 framebuffer/OAM identical for 600 frames.
NO MEDIDO: visual elimination of startup flashes and appearance on the user display.


2026-10-16 portrait follow-up: corrected host-list portrait exclusion introduced
in prior dialogue update. Host portraits now share text/backing placement and scale.
MEDIDO: both edges x 3 sizes x 7 zooms, host and OAM rendering identical with
expected portrait bounds; 158 tests passed, 4 ignored. No game logic or saves changed.


2026-10-16 independent dialogue UI layer (desktop, first migrated profile).
Native BG2/BG3 and portrait capture; world pass excludes them. GPU draws UI once
in physical window coordinates, independent of world zoom. Software and pause
paths included. FFTA_UI_LAYER=0 restores the preceding layout for comparison.
MEDIDO: 189 suite tests pass; separate RTX 5050 offscreen GPU/CPU comparison passes.
MEDIDO: EUR and USA extra=0 framebuffer/OAM equal for 600 frames each.
MEDIDO: real saved snapshot UI data and 72,855 opaque output pixels unchanged
at x1, x1.122462 and x2. Captures target/ui-layer-after-*.png.
NO MEDIDO: live user transitions and 8-character replay metric in this change.
Other menu profiles remain on existing paths; Android and installer code unchanged.


2026-10-16 dialogue continuation marker: measured USA replay frame 2141,
OBJ attrs 803B/0088/2122, tile 0x122, 8x16, palette 2, priority 0. Its graphics
match the existing EUR snapshot. Marker now shares dialogue layer, retaining
bounce and disabled state; tested with raw OAM and host replacements.
Text Bubbles moved to root directly after Fast Forward; eight rows fit.
MEDIDO: reconstructed before/after captures at x1/x2 confirm attachment.


2026-10-16 inward Size pivot: verified lower dialogue clipping in open window.
Native backing/portrait bounds choose upper/lower screen-edge anchor; Size grows
uniformly toward the middle, leaving space for the continuation marker without
following its bounce. Measured same live scene reconstructed at 110/200 percent.
100-percent image unchanged; EUR/USA extra=0 framebuffer/OAM equal for 600 frames.
192 tests pass, 5 ignored. Windows build: target/release/recomp-bubble-inward.exe.
User's existing game process was not closed; its in-use executable may remain old.

2026-10-16 independent action and hover-stat panels:
MEDIDO: actual EUR open-actions snapshot BG2=5C08/BG3=4A09; native left
information/portrait and right action list extracted into two separate layers.
Stats recognized from complete four-piece palette-11 OAM backing; glyph palette
12 retained. Native alpha blends against raw top BG, not world OBJ or an already
blended BG. Reused buffers include serial and parallel zoom rows.
Panels anchor to their screen corners and Size grows uniformly inward; world
zoom does not affect native UI size. Length remains dialogue-only.
MEDIDO: actual EUR action and USA stats snapshots reconstruct original hardware
framebuffer with changed_pixels=0, including Size=200 percent output captures:
target/panels-actions-after-200.png and target/panels-stats-after-200.png.
MEDIDO: 196 tests pass, 5 ignored; separate RTX 5050 GPU/CPU test passes.
MEDIDO: USA and EUR extra=0 framebuffer AND OAM equal for 600 frames each.
NO MEDIDO: new live user transitions and 1201-frame eight-character metric.
No sprite producer, installer logic, Android source, ROM or original save changed.
Windows executable: target/release/recomp-panels-corners.exe.
FFTA_UI_LAYER=0 retains old world UI for A/B comparison.

2026-10-16 panel Length: actions, portrait/info and hover stats now use the
persisted Text Bubbles Length for horizontal scale about their existing corner.
Size remains uniform scale; Length leaves vertical dimensions unchanged.
MEDIDO: 47 screen/recomp tests pass, 1 ignored; actual saved USA stats and EUR
actions reconstruct native UI with zero changed pixels at Length 150 percent.
MEDIDO: USA/EUR extra=0 framebuffer and OAM identical over 600 frames each.
Windows built: target/release/recomp-panels-length.exe; payload updated.
Captures: target/panels-actions-length150.png, target/panels-stats-length150.png.
NO MEDIDO: live user check. No sprite/game logic or installer source changed.

2026-10-16 Length correction: removed horizontal texture scaling for compact
panels. Capture native backing independently from glyphs/portraits/cursor.
Stats backing uses palette-11 pieces, excluding palette-12 glyph pieces.
Length translates foreground by an integer native offset toward screen center;
only the backing is extended using an interior column, keeping the outer cap.
Size still scales both axes equally; GPU/software/pause use the same packed UI.
MEDIDO: complete test suite passes, including rigid glyph translation, fixed
outer cap and exact Length=100 percent pixels. Actual saved EUR action/USA
stats native reconstruction: changed_pixels=0. Length=150 percent captures:
target/panels-actions-length-content150.png and target/panels-stats-length-content150.png.
MEDIDO: USA/EUR extra=0 framebuffer AND OAM identical for 600 frames each.
NO MEDIDO: live user transitions and new eight-character replay metric.
Windows: target/release/recomp-panels-length-content.exe; payload updated.
No guest logic, sprite producers, installer source or original saves changed.

2026-10-16 panel border completion:
MEDIDO: read-only live EUR snapshot PID 9292 at Length=200 percent shows action
header/footer and info strip cut short. BG2 action frame tiles 005..00A and
0A8..0B9 are decoration, not ordinary action labels. Include these in backing.
Background-only capture now ignores foreground world OBJ: portrait silhouettes
must not punch holes into the blue plane used for Length completion.
Use a fixed flat-side stretch seam, not one following each row's curve.
Separate native stats capture excludes legacy EUR host gap padding; original
cap geometry is retained in both orientations. Guest sprite producers unchanged.
MEDIDO: 201 tests passed, 5 ignored; tests cover action border retention,
background under portrait, mirrored rounded caps, and legacy host padding.
MEDIDO: saved native UI reconstruction zero changed pixels, including Length
100/200 and zoom x1/x2, Size100/150 percent. Before/after captures:
target/panels-current-window.png (actual original window),
target/panels-borders-actions-before.png / panels-borders-actions-after.png,
target/panels-borders-stats-before.png / panels-borders-stats-after.png,
target/panels-borders-actions-zoom2-size150.png (snapshot reconstructions).
MEDIDO: USA/EUR extra=0 framebuffer and OAM identical over 600 frames each.
NO MEDIDO: actual red enemy hover scene after fix; cap orientation regression
uses generated mirrored fixtures. No new 1201-frame character metric run.
Windows build target/release/recomp-panels-borders.exe; payload updated.
No installer source, original save, ROM or Android source modified. No commits.

2026-10-16 action-menu jitter:
Cause: full UI pixel bounds included animated OBJ cursor. Its varying left edge
changed Length displacement for the entire panel. Use static backing bounds for
Length distance and action panel anchoring. Cursor animation stays in foreground.
MEDIDO: saved real EUR menu snapshot with only cursor X changed to 171/173/175/177:
all UI rects identical; packed menu excluding cursor animation has identical SHA256
084a981ed32e54d7c077a13699e921167230f687534350701c698d120ddea7a7.
Captures target/menu-jitter-171.png through menu-jitter-177.png.
MEDIDO: 202 tests pass, 5 ignored; new regression includes cursor blink and Length
100/125/150/200 percent. USA/EUR extra=0 framebuffer AND OAM equal for 600 frames.
NO MEDIDO: live user validation after fix; live read-only sample had no matching
active menu cursor, so saved snapshot variants are a controlled reproduction.
Windows: target/release/recomp-menu-stable.exe; payload updated.
No installer source, guest producer, ROM or original save changes; no commits.

2026-10-16 target attack forecast in native screen-space UI:
MEDIDO: captured live EUR window PID6968 and six graphics/RAM regions read-only.
BG attacker signature 5C08/4A09 coexists with target OBJ backing 1F0/200/208.
Bug: begin_dialogue_frame detected Stats then unconditionally replaced it with
Actions. Target OAM remained in zoomed world. Preserve detected Stats beside BG
attacker info. Forecast glyphs share bank11: backing-only capture must identify
backing tiles, not the entire bank (glyph 228/230/238 must stay foreground).
MEDIDO: real EUR snapshot UI reconstruction changed_pixels=0 at zoom x1/x2;
UI layout rect identical, native target UI byte hash identical:
80F72591E91B9BC939864EFCEC4B82A1FEBFF6647AB755651B835EC5E3148841.
Captures target/forecast-live-window.png (before, actual window),
target/forecast-after-x1.png / forecast-after-x2.png (reconstructions),
target/forecast-after-size125-length200.png (Size/Length verification).
MEDIDO: 203 tests pass, 5 ignored, including both regional profiles, two zooms,
combined attacker BG and target OBJ, bank11 glyph exclusion, original OAM intact.
MEDIDO: USA/EUR extra=0 framebuffer and OAM equal for 600 frames each.
NO MEDIDO: actual USA target screenshot and live user verification after fix;
no new 1201-frame eight-character metric. No guest producer changes.
Windows target/release/recomp-attack-forecast.exe; payload updated.
No installer source, Android source, ROM or original save writes; no commits.

2026-10-16 experimental mouse, EUR language selector:
Read-only investigation plus opt-in semantic target layer (FFTA_MOUSE=1).
Five flag/text OAM groups and quill signature gate interaction; exact EUR SHA256
and short function bytes must match. Pointer0201C3D4 -> structure0200F3D4;
selection+2=0200F3D6, Thumb write0813FFCA. No guest memory/function modifications.
Hover navigates through ordinary Down pulses; click confirms once with A.
Targets follow existing viewport/zoom inverse transform. Per-frame virtual keys
recorded for replay. Manual buttons, pause/focus/scene loss cancel automation.
MEDIDO: all five target requests reached; one confirmation edge;206 tests pass,
5 ignored.700-frame interpreter and two native replay PPM files byte-identical,
hash d17fc597c1fb9325. USA/EUR extra=0 framebuffer/OAM equal for600 frames.
NO MEDIDO: physical mouse in user window, touch, other game scenes. Disabled by
default; no original save writes, ROM distribution, installer source or commits.
See docs/ffta_mouse_language.md. Windows target/release/recomp-mouse-language.exe.

2026-10-16 mouse direct selection:
User requested skipping intermediate languages. Measured direct selector-byte
commit updates the original quill/text palettes in two frames with no intervening
languages. Controller now commits just the validated selection byte between
emulated frames, then original game redraws; A confirmation remains one edge.
Exact EUR hash, short original bytes and complete current scene/pointer bounds
required. Recording adds FRAME mouse:INDEX; frames/runc/verify use guarded replay
and fail on scene/ROM mismatch. Historical key-only inputs remain compatible.
MEDIDO: all25 source/target pairs tested; real controller reaches4,2,0,3,1 with
no directional keys, no intermediate quill or palette state; one confirmation.
MEDIDO:208 suite tests pass,5 ignored. Interpreter/native full PPM byte equality
at all five destinations304/308/312/316/320;370-frame final also identical to
repeated native replay, hash d17fc597c1fb9325. Private target/mouse-direct-* files.
MEDIDO: USA/EUR extra=0 framebuffer AND OAM equal over600 frames without requests.
NO MEDIDO: user's physical mouse after direct-selection change. Experimental
FFTA_MOUSE=1 still required; only EUR language selector supported.
Windows target/release/recomp-mouse-direct.exe; payload updated. No original save,
ROM, installer source, Android or sprite producer changes; no commits.

2026-10-16 reusable invisible mouse grid:
Implemented interaction_grid.rs:8x8 logical cells across viewport, exact target
bounds within shared cells, EventId(context,option), layers/modal blockers,
cell tracking and hover entry/exit/click. Unchanged targets reuse the index.
EUR language adapter now routes through the grid; direct state commit unchanged.
MEDIDO:214 tests pass,5 ignored. Exact reference match at153600 test positions;
all38400 native language pixel centers match the previous target rectangles.
Actual EUR run:600 cells,1 build, five direct selections, one confirmation.
Five rendered images and input events byte-identical to direct-rectangle build.
Native/interpreter replay equal,370-frame hash d17fc597c1fb9325.
USA/EUR extra=0 framebuffer AND OAM identical over600 frames each.
NO MEDIDO: user's physical mouse with this build, new menu event adapters, any
speed gain over the five-option prototype. Main menu and battles not started.
Docs ffta_interaction_grid.md. Windows target/release/recomp-mouse-grid.exe.
No original save, ROM, installer source or Android modifications; no commits.

2026-10-16 EUR main menu mouse:
Reused invisible grid for Press Start, New Game, Saved Game, Color Mode,
Language, name keyboard/categories/Delete/OK and name Yes/No. Direct verified
selection bytes and original confirm/back keys; queued clicks cannot cross
menu contexts. Scene/ROM checks fail closed. FFTA_MOUSE=1 remains experimental.
MEDIDO: actual full-boot click scripts traverse the front-end, select TV,
Resume/Load and start New Game; letters/categories/name confirmation exercised.
Native1490-frame click replay equals interpreter pixels. USA/EUR extra=0
framebuffer AND OAM equal previous600-frame references.
NO MEDIDO: populated save-file slots/resume (all available saves blank), physical
mouse acceptance, USA mouse front-end and battle mouse controls.
Windows target/release/recomp-mouse-main.exe; docs/ffta_mouse_frontend.md.
No original save/ROM, installer source, Android or sprite-producer changes.
No commits.

2026-10-17 experimental battle hover:
Added ffta_battle_mouse.rs: reads logical stage dimensions, bounds, height/flags,
range styles and camera projection. Mouse movement queues a logical destination;
original cursor handler commits it. Left click confirms via original A after
commit; right click B. Exact USA/EUR ROM profiles and short code checks required.
FFTA_MOUSE=1 opt-in; action menus and unsupported cursor modes stay excluded.
MEDIDO:196 existing tutorial cells,20 range-style1; original projection matches
196/196 and range query256/256. Real USA full replay: three direct targets stay
exact in interpreter and Windows native32:9 zoom6; no intermediate arrow walk.
Single-frame wake pulse fixes an observed extra-step failure in the first build.
MEDIDO:221 tests pass,5 ignored; USA/EUR600-frame extra=0 framebuffer/OAM match.
Character metric11340..12540:1201 frames, exactly8 present, no duplicates.
EUR isolated original-function test passes on copied RAM; original scripted
mode4 was explicitly overridden only in that test, not in runtime behavior.
NO MEDIDO: full EUR manual hover, all stages/slopes, battle action menu mouse.
Windows target/release/recomp-mouse-battle.exe. docs/ffta_mouse_battle.md.
No original save/ROM, installer source, Android or sprite-producer edits.
No commits.

2026-10-17 battle input follow-up, awaiting user gameplay test:
Live vertical lists use their instance count/cursor rather than option names.
Original navigation commit USA080287A0 / EUR080299DC selects the requested row.
MEDIDO: USA replay jumps0->3->0, opens Action and Snowball; first one-frame wake
was missed by the menu polling cadence and replaced with wake-until-commit plus
suppression of repeated steps. Exact ROM and short function-byte checks gate it.
Follow-up adds plain left=A/right=B outside semantic targets, including dialogue
and forecasts, and fixes the field gate when unused BG layers are disabled.
Wait-facing arrows adapt mouse direction and queue original direction then A.
Name typing supports ASCII letters/digits, Backspace and Enter; original glyph
selection and keys insert the name. EUR3/USA2-page profiles are checked.
MEDIDO:10 battle-input tests and9 frontend/name tests pass; earlier filtered
FFTA suite37 pass,4 ignored. NO MEDIDO: user gameplay acceptance of follow-up.
Windows target/release/recomp.exe. No installer/save/ROM edits or commits.

2026-10-17 desktop mouse follow-up (experimental, off by default):
Input now has Keyboard / Gamepad / Mouse and Keyboard. The last uses its own
persisted ten-button profile (WASD, Q/E), preserving the Keyboard bindings.
The runtime and launcher select the active profile; assets/cursor.png is original.
MEDIDO: Do it/Cancel is a separate two-button handler, USA08029350/EUR0802A58C.
The adapter reads its live button records and drives original input, including
deferred A after the requested row. A controlled call selects Cancel successfully.
MEDIDO: world-map cursor logical stores USA080375CE/080376A2 and EUR08038B3A/
08038C0E, reached through the original input routine. Native replay matches the
interpreter framebuffer; requested native screen points remain exact after release.
Town window lists and nested shop/pub text lists use live row counts/geometry.
Two cursor styles are handled. Polling lists release each direction after an
observed step because the pub list consumes edges instead of repeating held input.
MEDIDO: extra=0 framebuffer and OAM identical in600 frames against the previous
executable; tutorial regression8 present/8 drawn in all1201 final frames, no duplicates.
NO MEDIDO: every mission/inventory paging state and manual clicks at every scale.
Pending: reported end-turn facing issue and repeated thunder/effects in32:9.
Android port is deferred; mouse remains off by default there. No installer,
original ROM/save or sprite/PPU renderer edits. No commits. docs/ffta_mouse_world.md.
