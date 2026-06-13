# Forza Horizon 6 Reverb Templates Documentation

The file `media/Audio/Reverb_Templates.xml` defines the audio environment presets (reverb templates) used by the game engine. Each `<Template>` represents a specific physical environment or zone (tunnels, canyons, overpasses, open tracks) and dictates how the audio system mixes the **Dry** (unprocessed/direct) and **Wet** (reverberated/echoed) signals.

---

## 📂 Template Attributes Reference

Every template defines the following properties:

| Attribute | Unit/Type | Description |
| :--- | :--- | :--- |
| `Name` | String | Unique identifier for the environment zone (e.g., `12_PGTunnel`, `0_TrackOpen`). |
| `TrackReflectionDist` | Float | Base distance scaling for early sound reflections. |
| `NearDist` / `FarDist` | Meters | Distance boundaries defining the transition from close-up audio mixing to distant audio mixing. |
| `NearDry` / `FarDry` | Float (Scale) | Multiplier for the dry (direct) signal volume at close/far distances. Normally `1.0` (0 dB attenuation). |
| `NearWet` / `FarWet` | Float (Scale) | Multiplier for the wet (reverb) signal volume at close/far distances. `0.0` is completely dry, `1.0` is fully wet. |
| `DryLevelmB` | Millibels (mB) | Direct signal volume level offset. `0` mB represents 0 dB (no change). |
| `RoomLevelmB` | Millibels (mB) | Overall room reflections level. Lower values (e.g., `-800` mB = -8.0 dB) reduce the room's volume. |
| `RoomHFLevelmB` | Millibels (mB) | Room high-frequency attenuation. Simulates sound absorption of high pitches. |
| `RoomRolloffFactor` | Float | The rate at which the room reverb volume decays over distance. |
| `DecayTimeSec` | Seconds | How long the late reverb tail takes to decay. (e.g., `4.0` in canyons vs `1.2` in open tracks). |
| `DecayHFRatio` | Float | Ratio of high-frequency decay time relative to low-frequency decay. |
| `ReflectionsLevelmB` | Millibels (mB) | Volume level of early discrete sound reflections (echoes). |
| `ReflectionsDelaySec` | Seconds | Time delay between the direct sound and early reflections. |
| `ReverbLevelmB` | Millibels (mB) | Volume level of late/diffuse reverb tail reflections. |
| `ReverbDelaySec` | Seconds | Time delay between early reflections and late reverb tail. |
| `DiffusionPercent` | Percent (0-100)| Density/texture of the late reverb tail. |
| `DensityPercent` | Percent (0-100)  | Density of the early reflections. |
| `HFReferenceHz` | Hertz (Hz) | Reference frequency used for high-frequency calculations. |
| `SnapshotMix` | String | FMOD mixer snapshot to load (e.g., `InTunnel` vs `NotInTunnel`). |

---

## 🎛️ Tweaking and "Cheesing" Options

By modifying these XML parameters, you can alter how engines sound inside tunnels or open areas.

### 1. The "Clean & Dry" Cheesing (Disable Reverb Completely)
If you dislike the echoing effect in tunnels or canyons and want a pure engine note everywhere:
* **How to do it**: Set `NearWet="0.000000"` and `FarWet="0.000000"` on all templates.
* **Result**: The game will not apply any reverb/echo DSP effects, leaving the engine audio clean and dry.

### 2. The "Echo Tunnel Everywhere" Cheesing
If you want the dramatic, echo-heavy tunnel sound in the open world:
* **How to do it**: Replace the parameters of `0_TrackOpen` (or other open areas) with the parameters of `12_PGTunnel`.
* **Result**: The game will simulate a concrete tunnel sound even in wide-open fields.

### 3. Boosting Engine Volume in Tunnels/Canyons
If you find that the engine sound gets too quiet or gets muffled inside tunnels/canyons due to wet mix absorption:
* **How to do it**: 
  * Increase `NearDry` and `FarDry` above `1.0` (e.g., `1.5` or `2.0`).
  * Increase `DryLevelmB` to positive millibel values (e.g., `300` mB = +3 dB, `600` mB = +6 dB).
* **Result**: Direct engine sound volume is boosted when entering those zones, making the engine scream noticeably louder inside tunnels.

### 4. Making Tunnels Sound Like Huge Canyons
To make a short tunnel sound like a massive mountain cavern:
* **How to do it**: Increase `DecayTimeSec` to `4.0` or `5.0` seconds and set `ReverbLevelmB` to a higher value (e.g., `1200` mB).
* **Result**: The engine echo will linger for much longer before fading away.
