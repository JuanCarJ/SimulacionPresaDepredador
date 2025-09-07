// src/main.rs

// Este archivo es el "frontend" o visualizador de la simulación.
// Utiliza la librería macroquad para dibujar el estado del "backend" (el módulo de simulación).
// Su responsabilidad es pintar, no ejecutar la lógica de las reglas del ecosistema.

use macroquad::prelude::*;
// Declara los otros módulos para que `main` pueda usarlos.
mod entidades;
mod simulacion;

/// Dibuja una leyenda en la esquina superior derecha para identificar los colores.
fn dibujar_leyenda() {
    let x_offset = screen_width() - 150.0;
    let y_offset = 20.0;
    let rect_size = 15.0;
    let text_offset = rect_size + 5.0;
    let text_color = DARKGRAY;
    let font_size = 18.0;

    // Leyenda Conejo
    draw_circle(x_offset + rect_size / 2.0, y_offset + rect_size / 2.0, rect_size / 2.0, WHITE);
    draw_text("Conejo", x_offset + text_offset, y_offset + rect_size / 2.0 + font_size / 2.0 - 5.0, font_size, text_color);

    // Leyenda Cabra
    draw_circle(x_offset + rect_size / 2.0, y_offset + rect_size / 2.0 + rect_size + 10.0, rect_size / 2.0, BROWN);
    draw_text("Cabra", x_offset + text_offset, y_offset + rect_size / 2.0 + rect_size + 10.0 + font_size / 2.0 - 5.0, font_size, text_color);
}


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
    
    // Dibuja al depredador, cambiando de color según su estado de alimentación.
    if sim.depredador.vivo {
        let depredador_color = if sim.depredador.reserva_comida_kg >= entidades::DEPREDADOR_CONSUMO_OPTIMO_DIARIO_KG {
            RED // Óptimo
        } else if sim.depredador.reserva_comida_kg >= entidades::DEPREDADOR_CONSUMO_MINIMO_DIARIO_KG {
            ORANGE // Mínimo
        } else {
            DARKGRAY // Peligro de muerte
        };
        draw_circle(screen_width() / 2.0, 50.0, 20.0, depredador_color);
    }

    // Muestra las estadísticas de la simulación como texto.
    let font_size = 20.0;
    let mut current_y = 20.0;

    // Información general
    draw_text(&format!("Día: {}", sim.dia), 10.0, current_y, font_size, DARKGRAY);
    current_y += 25.0;

    // Conteo de especies
    let (conejos, cabras) = sim.contar_especies();
    draw_text(&format!("Conejos: {}", conejos), 10.0, current_y, font_size, DARKGRAY);
    current_y += 25.0;
    draw_text(&format!("Cabras: {}", cabras), 10.0, current_y, font_size, DARKGRAY);
    current_y += 25.0;
    draw_text(&format!("Población Total: {}", sim.presas.len()), 10.0, current_y, font_size, DARKGRAY);
    current_y += 25.0;


    // Estado del depredador
    draw_text(&format!("Reserva Depredador: {:.1} kg", sim.depredador.reserva_comida_kg), 10.0, current_y, font_size, DARKGRAY);
    current_y += 25.0;

    if sim.depredador.vivo {
        let estado_depredador = if sim.depredador.reserva_comida_kg >= entidades::DEPREDADOR_CONSUMO_OPTIMO_DIARIO_KG {
            "Estado: Óptimo"
        } else if sim.depredador.reserva_comida_kg >= entidades::DEPREDADOR_CONSUMO_MINIMO_DIARIO_KG {
            "Estado: Mínimo"
        } else {
            "Estado: Peligro"
        };
        draw_text(estado_depredador, 10.0, current_y, font_size, DARKGRAY);
    }


    // Muestra un mensaje de fin de juego si el depredador muere.
    if !sim.depredador.vivo {
        let texto_fin = "¡EL DEPREDADOR HA MUERTO!";
        let text_dims = measure_text(texto_fin, None, 40, 1.0);
        draw_text(texto_fin, screen_width() / 2.0 - text_dims.width / 2.0, screen_height() / 2.0, 40.0, BLACK);
    }
     // Muestra un mensaje si las presas se extinguen.
     if sim.presas.is_empty() && sim.depredador.vivo {
        let texto_fin = "¡LAS PRESAS SE HAN EXTINGUIDO!";
        let text_dims = measure_text(texto_fin, None, 40, 1.0);
        draw_text(texto_fin, screen_width() / 2.0 - text_dims.width / 2.0, screen_height() / 2.0, 40.0, BLACK);
    }

    // Dibuja la leyenda al final para que esté en primer plano.
    dibujar_leyenda();
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