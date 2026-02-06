# OMMP - Terminal Music Player

## Overview

OMMP (Oh My Music Player)는 터미널 기반 독립형 음악 플레이어입니다. [rmpc](https://github.com/mierak/rmpc)에서 영감을 받았지만, MPD 클라이언트가 아닌 독립 실행형으로 FLAC/MP3/M4A/OGG/WAV 파일을 직접 재생합니다.

## Tech Stack

| 역할 | 크레이트 | 버전 |
|------|---------|------|
| TUI 렌더링 | `ratatui` | 0.29 |
| 터미널 제어 | `crossterm` | 0.28 |
| 오디오 재생 | `rodio` + symphonia | 0.21 |
| 메타데이터 | `lofty` | 0.21 |
| 파일 스캔 | `walkdir` | 2 |
| 채널 통신 | `crossbeam-channel` | 0.5 |
| 에러 처리 | `anyhow` | 1 |
| 랜덤 | `rand` | 0.8 |

## Architecture

### Thread Model

```
Main Thread (UI + State)     Input Thread          Audio Thread
        │                        │                      │
        │←── Event::Key ─────────│                      │
        │←── Event::Mouse ───────│                      │
        │←── Event::Tick ────────│  (200ms interval)    │
        │                                               │
        │─── PlayerCommand::Play ──────────────────────→│
        │─── PlayerCommand::Pause ─────────────────────→│
        │─── PlayerCommand::Seek ──────────────────────→│
        │←── AudioEvent::PositionUpdate ────────────────│
        │←── AudioEvent::TrackFinished ─────────────────│
```

- **Main Thread**: UI 렌더링 + 상태 관리. `App` 구조체가 모든 상태를 소유
- **Input Thread**: crossterm 이벤트 폴링, `Event` 채널로 전달
- **Tick Thread**: 200ms 간격으로 `Event::Tick` 발생 (호버 갱신, UI 리프레시)
- **Audio Thread**: rodio `OutputStream`+`Sink` 소유, 명령 수신 + 위치 보고

### Action-Based State Mutation

모든 상태 변경은 `AppAction` 열거형을 통해 `App::handle_action()`에서 처리됩니다.

```rust
pub enum AppAction {
    Quit, PlayTrack(usize), PauseResume, Stop,
    NextTrack, PrevTrack,
    SetVolume(f32), VolumeUp, VolumeDown,
    Seek(f64), SeekForward, SeekBackward,
    ToggleShuffle, CycleRepeat,
    SwitchTab(Tab), FocusNext, FocusPrev, FocusPane(FocusedPane),
    AddToQueue(Vec<usize>), ClearQueue, RemoveFromQueue(usize),
    PlayQueueIndex(usize), SetQueueSelection(usize),
    UpdatePosition { position_secs, duration_secs },
    TrackFinished,
    SearchQuery(String), EnterSearchMode, ExitSearchMode,
}
```

## Project Structure

```
src/
  main.rs                 — Entry point, terminal setup, event loop
  app/
    mod.rs                — App struct, AppAction enum, handle_action()
    handler.rs            — Event dispatch, mouse hit-testing, hover logic
    state.rs              — PlaybackState, QueueState, Tab, FocusedPane, RepeatMode
  audio/
    mod.rs                — Re-exports
    player.rs             — AudioEngine, PlayerCommand, audio thread
  library/
    mod.rs                — Library struct, query methods
    scanner.rs            — walkdir-based recursive file scanner
    track.rs              — Track struct, metadata extraction via lofty
  ui/
    mod.rs                — Ui struct, owns all panes/widgets, render()
    layout.rs             — LayoutAreas computation
    theme.rs              — Theme struct (colors, styles)
    pane.rs               — Pane trait definition
    panes/
      mod.rs
      library_pane.rs     — Queue tab: track list browser
      dir_browser_pane.rs — Directories tab: filesystem navigation
      queue_pane.rs       — Center column: playback queue
      artists_pane.rs     — Artists tab
      album_artists_pane.rs — Album Artists tab
      albums_pane.rs      — Albums tab
      genre_pane.rs       — Genre tab
      playlists_pane.rs   — Playlists tab
      search_pane.rs      — Search tab with text input
      lyrics_pane.rs      — Right column: lyrics display
    widgets/
      mod.rs
      status_bar.rs       — 4-section status bar
      progress_bar.rs     — Playback progress bar
      tab_bar.rs          — Category tab bar with hover highlight
  event/
    mod.rs                — Event, AudioEvent enums
    input.rs              — Input thread, Tick thread spawners
```

## UI Layout

```
┌──────────────────┬──────────────────┬─────────────┬──────────────┐
│ [▶ Playing]      │ [1:57/4:15]      │ Song - Artist│ [Vol:80% ⇆ ↻]│ ← Status Bar (4 rows)
│                  │ (119kbps)        │ - Album      │              │
├──────────────────┴──────────────────┴─────────────┴──────────────┤
│     Queue │ Directories │ Artists │ Album Artists │ Albums │ ...  │ ← Tab Bar (3 rows)
├─────────────┬──────────────────┬─────────────────────────────────┤
│             │                  │                                 │
│  Library    │   Queue          │          Lyrics                 │ ← Dashboard
│  (30%)      │   (35%)          │          (35%)                  │
│             │                  │                                 │
├─────────────┴──────────────────┴─────────────────────────────────┤
│  ▶  ╭━━━━━━━━━━━━━━━━━━━━━━━━━━━━━╮  1:57 / 4:15               │ ← Progress Bar (3 rows)
└─────────────────────────────────────────────────────────────────────┘
```

### Layout Computation (LayoutAreas)

- **Vertical**: StatusBar(4) | TabBar(3) | Dashboard(Min 10) | ProgressBar(3)
- **Dashboard Horizontal**: Library(30%) | Playlist(35%) | Lyrics(35%)

### Tab-to-Pane Mapping

왼쪽 컬럼은 현재 탭에 따라 다른 Pane을 표시합니다:

| Tab | Left Pane | Center Pane | Right Pane |
|-----|-----------|-------------|------------|
| Queue | LibraryPane | QueuePane | LyricsPane |
| Directories | DirBrowserPane | QueuePane | LyricsPane |
| Artists | ArtistsPane | QueuePane | LyricsPane |
| Album Artists | AlbumArtistsPane | QueuePane | LyricsPane |
| Albums | AlbumsPane | QueuePane | LyricsPane |
| Genre | GenrePane | QueuePane | LyricsPane |
| Playlists | PlaylistsPane | QueuePane | LyricsPane |
| Search | SearchPane | QueuePane | LyricsPane |

## Pane Trait

모든 Pane은 공통 트레이트를 구현합니다:

```rust
pub trait Pane {
    fn render(&mut self, frame: &mut Frame, area: Rect, focused: bool, app: &App, theme: &Theme);
    fn handle_key(&mut self, key: KeyEvent, app: &App) -> Option<AppAction>;
    fn handle_mouse(&mut self, event: MouseEvent, area: Rect, app: &App) -> Option<AppAction>;
    fn handle_scroll(&mut self, up: bool, app: &App) -> Option<AppAction> { None }
}
```

### Pane Features

각 리스트형 Pane은 다음을 포함합니다:
- `selected: usize` — 현재 선택된 항목 인덱스
- `scroll_offset: usize` — 스크롤 오프셋
- `hover_row: Option<usize>` — 마우스 호버 중인 행
- Auto-scroll: `render()` 내에서 `selected`가 보이도록 `scroll_offset` 자동 조정
- Scrollbar: 항목 수가 표시 영역 초과 시 `Scrollbar` 위젯 표시
- Hover highlight: `Color::Indexed(238)` 배경으로 호버 행 강조

## Keyboard Bindings

| Key | Action |
|-----|--------|
| `q` | Quit |
| `Space` | Play/Pause toggle |
| `1`-`8` | Switch tab |
| `Tab` / `Shift+Tab` | Cycle focus (Library → Playlist → Lyrics) |
| `h` / `l` | Focus left/right pane |
| `j` / `k` or `↓` / `↑` | Navigate list items |
| `g` / `G` | Jump to first/last item |
| `Enter` | Select item (add to queue / enter directory) |
| `Backspace` | Go to parent directory (Directories tab) |
| `n` / `N` | Next/Previous track |
| `+` / `-` | Volume up/down (5% step) |
| `→` / `←` | Seek forward/backward (5 seconds) |
| `s` | Toggle shuffle |
| `r` | Cycle repeat mode (Off → All → One) |
| `d` | Remove selected from queue |
| `/` or `i` | Enter search mode (Search tab) |
| `Esc` | Exit search mode |

## Mouse Support

마우스 동작에는 `\x1b[?1003h` SGR any-event 모드가 활성화되어 있습니다.

| Action | Behavior |
|--------|----------|
| Hover on pane | Focus switches (border turns cyan) |
| Hover on tab | Tab text highlights in cyan (visual only) |
| Click on pane item | Select item |
| Double-click on track | Immediately play track |
| Click on tab | Switch to that tab |
| Click on progress bar | Seek to position |
| Scroll wheel | Scroll list (3 items per tick) |

### Hover System

- `mouse_pos: Option<(u16, u16)>` — Ui에 저장된 마지막 마우스 위치
- `hovered_tab: Option<usize>` — 현재 호버된 탭 인덱스
- `hover_row: Option<usize>` — 각 Pane의 호버 행
- `refresh_hover()` — Tick 이벤트마다 호출, 마우스 위치로 Pane 포커스 + 행 호버 갱신
- `update_hover()` — 탭별로 올바른 Pane에 hover_row 설정
- `clear_all_hovers()` — 모든 Pane의 hover_row를 None으로 리셋

## Theme

기본 테마 (`Theme::default()`) — rmpc 스타일:

| Element | Color |
|---------|-------|
| Focused border | Cyan |
| Unfocused border | DarkGray |
| Selection highlight | Cyan bg + Black fg + Bold |
| Active tab | Cyan + Bold |
| Inactive tab | DarkGray |
| Hovered tab | Cyan (no bold) |
| Progress bar fill | Cyan |
| Playing indicator | Green |
| Title | White + Bold |
| Artist | Gray |
| Hover row bg | Color::Indexed(238) |

## Library

### Scanning

- `walkdir`로 `~/Music` 재귀 탐색
- 지원 확장자: `.flac`, `.mp3`, `.m4a`, `.mp4`, `.ogg`, `.wav`
- 백그라운드 스레드에서 스캔, 완료 시 `App.library` 교체

### Track Metadata

`lofty`를 통해 추출:
- title, artist, album, album_artist, genre
- track_number, duration, bitrate (kbps)
- lyrics (embedded)

### Query Methods

```rust
get_artists() -> Vec<String>
get_album_artists() -> Vec<String>
get_genres() -> Vec<String>
get_albums() -> Vec<(String, String)>  // (album, artist)
get_tracks_by_artist(artist) -> Vec<usize>
get_tracks_by_album_artist(album_artist) -> Vec<usize>
get_tracks_by_album(album) -> Vec<usize>
get_tracks_by_genre(genre) -> Vec<usize>
get_albums_by_album_artist(album_artist) -> Vec<String>
get_directory_entries(dir) -> (Vec<String>, Vec<usize>)
search(query) -> Vec<usize>  // case-insensitive substring match
```

## Audio Engine

- `rodio 0.21` + `symphonia-all` feature로 FLAC/MP3/M4A/OGG/WAV 디코딩
- `PlayerCommand` 열거형: Play, Pause, Resume, Stop, SetVolume, Seek
- `AudioEvent` 열거형: PositionUpdate, TrackFinished, TrackError, Playing, Paused, Stopped
- OutputStream은 오디오 스레드 내에서 생성 및 유지

## Queue Management

- `QueueState`: tracks (인덱스 목록), current_index, selected_index, scroll_offset
- Add: 트랙 인덱스를 큐 끝에 추가, 빈 큐였으면 자동으로 current_index=0
- Remove: 인덱스 제거 후 current_index 보정
- Next: 순서 재생 / 셔플(랜덤) / 반복(All: 처음으로, One: 같은 트랙)
- Previous: 3초 이상 재생 시 처음부터 재시작, 아니면 이전 트랙

## Status Bar Widget

4-section 가로 분할 (│ 구분):

```
[▶ Playing] │ [1:57/4:15 (119kbps)] │ [Song - Artist - Album] │ [Vol:80% ⇆ ↻]
```

- Section 1: 재생 상태 아이콘 (▶/⏸/⏹) + 라벨
- Section 2: 현재 위치/전체 시간 + 비트레이트
- Section 3: 제목 - 아티스트 - 앨범
- Section 4: 볼륨 + 셔플(⇆) + 반복(↻/↻1), 활성 시 Cyan, 비활성 시 DarkGray

## Progress Bar Widget

Rounded style:

```
▶  ╭━━━━━━━━━━━━━━━╮  1:57 / 4:15
```

- 왼쪽: 재생 상태 아이콘
- 가운데: ratatui Gauge (Cyan 채움)
- 오른쪽: 현재 시간 / 전체 시간

## Build & Run

```bash
cargo build --release
cargo run --release
```

기본 음악 디렉토리: `~/Music` (없으면 현재 디렉토리 사용)
