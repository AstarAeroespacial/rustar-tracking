import urllib.request
from datetime import datetime, timedelta, timezone

import matplotlib.pyplot as plt
import pandas as pd
from skyfield.api import EarthSatellite, load, wgs84

print(" ============== VALIDACIÓN DOPPLER - SATÉLITE ================ ")

# 1. LEER TLE
print("\n[1] Leyendo TLE...")
try:
    with open("validacion_doppler/satelites/satelite_tle.txt", "r") as f:
        lines = f.readlines()

    tle_name = lines[0].strip()
    tle_line1 = lines[1].strip()
    tle_line2 = lines[2].strip()

    print(f"✓ {tle_name}")

except FileNotFoundError:
    print("✗ No se encontró validacion_doppler/satelites/satelite_tle.txt")
    print("Ejecuta primero: cargo run --example track_satelite")
    exit(1)

# 2. CONFIGURAR
satellite = EarthSatellite(tle_line1, tle_line2, tle_name)
buenos_aires = wgs84.latlon(-34.6037, -58.3816, elevation_m=25)
ts = load.timescale()

print(f"✓ Observador: Buenos Aires")

# 3. COMPARAR CON RUST
print("\n[2] Comparando con Rust...")

try:
    df_rust = pd.read_csv("validacion_doppler/satelites/doppler_output.csv")
    df_rust["timestamp"] = pd.to_datetime(df_rust["timestamp"])

    resultados = []
    difference = satellite - buenos_aires
    c = 299792458.0  # velocidad de la luz (m/s)
    freq = 145.96e6  # 145.96 MHz

    print("Calculando", end="", flush=True)

    for idx, row in df_rust.iterrows():
        dt = row["timestamp"].to_pydatetime()
        if dt.tzinfo is None:
            dt = dt.replace(tzinfo=timezone.utc)

        t_sky = ts.from_datetime(dt)
        t_sky2 = ts.from_datetime(dt + timedelta(seconds=10))

        # Calcular range rate
        topocentric = difference.at(t_sky)
        _, _, distance = topocentric.altaz()

        topocentric2 = difference.at(t_sky2)
        _, _, distance2 = topocentric2.altaz()

        range_rate = ((distance2.km - distance.km) * 1000) / 10.0  # m/s

        # Calcular Doppler
        doppler_skyfield = -freq * (range_rate / c)
        doppler_rust = row["doppler_145.96MHz_Hz"]

        resultados.append(
            {
                "timestamp": row["timestamp"],
                "doppler_skyfield": doppler_skyfield,
                "doppler_rust": doppler_rust,
            }
        )

    df_comp = pd.DataFrame(resultados)
    df_comp["diff_doppler"] = (
        df_comp["doppler_rust"] - df_comp["doppler_skyfield"]
    ).abs()

    print("\n============= RESULTADO ============")

    print(f"\nDoppler @ 145.96 MHz:")
    print(
        f"  Diferencia promedio:  {df_comp['diff_doppler'].mean():>10.2f} Hz"
    )
    print(f"  Diferencia máxima:    {df_comp['diff_doppler'].max():>10.2f} Hz")
    print(f"  Diferencia std:       {df_comp['diff_doppler'].std():>10.2f} Hz")

    # Evaluación
    print("\n=========== EVALUACIÓN =============\n")

    diff_mean = df_comp["diff_doppler"].mean()
    if diff_mean < 2:
        print(f"✓ EXCELENTE: {diff_mean:.2f} Hz (< 2 Hz)")
    elif diff_mean < 10:
        print(f"✓ BUENO: {diff_mean:.2f} Hz (< 10 Hz)")
    elif diff_mean < 50:
        print(f"⚠ ACEPTABLE: {diff_mean:.2f} Hz (< 50 Hz)")
    else:
        print(f"✗ REVISAR: {diff_mean:.2f} Hz (> 50 Hz)")

    # ═══════════════════════════════════════════════════════════════════
    # GRÁFICOS DE VALIDACIÓN
    # ═══════════════════════════════════════════════════════════════════
    # Estos gráficos comparan los cálculos de Doppler entre:
    #   - Rust (nuestra implementación)
    #   - Skyfield (referencia Python validada)
    #
    # Doppler shift a lo largo del tiempo
    #   - Línea sólida azul: Rust
    #   - Línea punteada naranja: Skyfield
    #   - Si las líneas se superponen casi completamente = implementación correcta

    fig, ax = plt.subplots(figsize=(14, 6))

    # Doppler comparación
    ax.plot(
        df_comp["timestamp"].values,
        (df_comp["doppler_rust"] / 1000).values,
        label="Rust",
        linewidth=2,
    )
    ax.plot(
        df_comp["timestamp"].values,
        (df_comp["doppler_skyfield"] / 1000).values,
        label="Skyfield (referencia)",
        linewidth=2,
        linestyle="--",
        alpha=0.7,
    )
    ax.set_ylabel("Doppler Shift (kHz)", fontsize=11)
    ax.set_xlabel("Tiempo", fontsize=11)
    ax.set_title(
        f"Validación Doppler {tle_name} - Diferencia promedio: {diff_mean:.2f} Hz",
        fontsize=13,
        fontweight="bold",
    )
    ax.legend()
    ax.grid(True, alpha=0.3)

    plt.tight_layout()
    plt.savefig(
        "validacion_doppler/satelites/validacion_doppler.png",
        dpi=150,
        bbox_inches="tight",
    )
    print(f"\n✓ Gráfico: validacion_doppler/satelites/validacion_doppler.png")
    print("\nCÓMO INTERPRETAR EL GRÁFICO:")
    print("  Rust vs Skyfield superpuestos → las curvas deben coincidir")
    print(
        "  Diferencia promedio: objetivo < 5 Hz (excelente), < 50 Hz (aceptable)"
    )

except FileNotFoundError:
    print("✗ No se encontró validacion_doppler/satelites/doppler_output.csv")
    print("\nEjecuta primero: cargo run --example track_satelite")
