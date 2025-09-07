// src/main.rs

// Este archivo es el "frontend" o visualizador de la simulación.
// Utiliza la librería macroquad para dibujar el estado del "backend" (el módulo de simulación).
// Su responsabilidad es pintar, no ejecutar la lógica de las reglas del ecosistema.

use macroquad::prelude::*;
// Declara los otros módulos para que `main` pueda usarlos.
mod entidades;
mod simulacion;

/// Dibuja el estado actual de la simulación en la pantalla.
fn dibujar_simulacion(sim: &simulacion::Simulacion) {
    clear_background(Color::from_rgba(135, 206, 235, 255)); // Sky Blue

    // Dibuja cada presa en la simulación.
    for presa in &sim.presas {
        // El color depende de la especie.
        let color = match presa.especie() {
            entidades::Especie::Conejo => WHITE,
            entidades::Especie::Cabra => BROWN,
        };
        
        // Genera una posición consistente usando el ID para que no salten por la pantalla.
        let mut x = (presa.id() * 27) as f32 % (screen_width() - 40.0) + 20.0;
        let mut y = (presa.id() * 53) as f32 % (screen_height() - 120.0) + 100.0;
        
        // Añade un pequeño movimiento basado en la edad para que no se apilen.
        x = (x + presa.edad() as f32 * 0.1) % (screen_width() - 40.0) + 20.0;
        y = (y + presa.edad() as f32 * 0.1) % (screen_height() - 120.0) + 100.0;

        // El radio del círculo es proporcional al peso de la presa.
        let radio = 4.0 + (presa.peso() / 15.0) as f32;
        draw_circle(x, y, radio, color);
    }
    
    // Dibuja al depredador como un círculo rojo en la parte superior.
    if sim.depredador.vivo {
        draw_circle(screen_width() / 2.0, 50.0, 20.0, RED);
    }

    // Muestra las estadísticas de la simulación como texto.
    let font_size = 24.0;
    let texto_dia = format!("Día: {}", sim.dia);
    let texto_presas = format!("Población: {}", sim.presas.len());
    let texto_comida = format!("Reserva Depredador: {:.1} kg", sim.depredador.reserva_comida_kg);
    
    draw_text(&texto_dia, 10.0, 20.0, font_size, DARKGRAY);
    draw_text(&texto_presas, 10.0, 45.0, font_size, DARKGRAY);
    draw_text(&texto_comida, 10.0, 70.0, font_size, DARKGRAY);

    // Muestra un mensaje de fin de juego si el depredador muere.
    if !sim.depredador.vivo {
        let texto_fin = "¡EL DEPREDADOR HA MUERTO!";
        let text_dims = measure_text(texto_fin, None, 40, 1.0);
        draw_text(texto_fin, screen_width() / 2.0 - text_dims.width / 2.0, screen_height() / 2.0, 40.0, BLACK);
    }
     // Muestra un mensaje si las presas se extinguen.
     if sim.presas.is_empty() && sim.depredador.vivo{
        let texto_fin = "¡LAS PRESAS SE HAN EXTINGUIDO!";
        let text_dims = measure_text(texto_fin, None, 40, 1.0);
        draw_text(texto_fin, screen_width() / 2.0 - text_dims.width / 2.0, screen_height() / 2.0, 40.0, BLACK);
    }
}

/// Punto de entrada de la aplicación, marcado para ser ejecutado por macroquad.
#[macroquad::main("Simulador de Ecosistema")]
async fn main() {
    // Se crea la instancia de la simulación una sola vez.
    let mut sim = simulacion::Simulacion::new();
    let mut tiempo_desde_ultimo_dia = 0.0;
    
    // Bucle principal que se ejecuta en cada fotograma.
    loop {
        // Permite controlar la velocidad de la simulación con las teclas de flecha.
        let tiempo_por_dia = if is_key_down(KeyCode::Right) {
            0.02 // Cámara rápida
        } else if is_key_down(KeyCode::Left) {
            0.5  // Cámara lenta
        } else {
            0.1  // Velocidad normal (10 días por segundo)
        };

        // Acumula el tiempo transcurrido desde el último fotograma.
        tiempo_desde_ultimo_dia += get_frame_time();
        
        // Si ha pasado suficiente tiempo, avanza la simulación un día.
        if tiempo_desde_ultimo_dia > tiempo_por_dia {
            sim.avanzar_dia();
            tiempo_desde_ultimo_dia = 0.0;
        }

        // Dibuja el estado actual.
        dibujar_simulacion(&sim);
        
        // Espera al siguiente fotograma.
        next_frame().await
    }
}