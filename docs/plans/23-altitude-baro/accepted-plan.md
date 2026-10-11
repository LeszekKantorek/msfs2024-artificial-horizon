# Plan implementacji #23 — wysokość barometryczna i nastawa ciśnienia

## Cel i przyjęte decyzje

Dostarczyć kompletną ścieżkę Rust → SSE → mobilny PFD dla [zadania #23](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/23), wykorzystując istniejący układ, warstwy SVG i harmonogram renderowania.

* Taśma pokazuje 400 ft: ±200 ft względem nieruchomego wskaźnika, podziałkę co 20 ft i etykiety co 100 ft.
* Uzgodniony zakres odczytu: **−9 999…99 999 ft**. Poza nim widoczne `OVR`, bez ograniczania wartości telemetrii.
* Uzgodniony bęben dolnych cyfr: **00/20/40/60/80**. Przesunięcie wynika z pełnej wartości próbki.
* Ciśnienie pod wysokościomierzem: inHg, dwie cyfry po przecinku, wyłącznie odczyt.
* Podziałka i jednostka ciśnienia odpowiadają sekcjom 3.3.4–3.3.5 [instrukcji Garmin Rev. 2](https://static.garmin.com/pumac/190-01112-12_02.pdf). Grafika pozostaje własna.
* Wybrana wysokość, alerty i VSI pozostają w #24–#25. Nie dodawać ich pól ani atrap do strony produkcyjnej.

## Implementacja

### 1. Rozszerzyć kontrakt danych
**Umiejętności: `context-apply`, `codebase-design`.**

* Dodać opcjonalne pola v1 `altitude_ft` i `baro_inhg`.
* Wprowadzić typy Rust `BarometricAltitude` — dowolna skończona liczba ze znakiem — oraz `BarometricSetting` — skończona liczba większa od zera.
* Rozszerzyć `FlightSample` metodą `with_altitude(Option<BarometricAltitude>, Option<BarometricSetting>)`, zachowując obecny konstruktor.
* Przenieść pola przez aktualny stan i serializację snapshotu. Pomijać nieobecne wartości oraz rozszerzenia poza stanem `live`; publikacja attitude-only musi czyścić poprzednie wartości.
* Normalizować oba pola niezależnie w JavaScript do liczby albo `null`. Zachować wersję schematu, endpoint, kolejność callbacków i wspólny termin ważności próbki.
* Wysokość oznacza gotowe wskazanie dla podanej nastawy. Przeglądarka nie przelicza wysokości z ciśnienia.

### 2. Rozszerzyć deterministyczne demo
**Umiejętności: `codebase-design`, `designing-tests`.**

* Zachować obecny 18-sekundowy cykl, częstotliwość 20 Hz oraz przebiegi istniejących instrumentów.
* Dodać sześć trzysekundowych scenariuszy wysokości: −120→120, 980→1020, 9980→10020, 10020→9980, 1020→980 i 120→−120 ft.
* Obliczać przebieg liniowo z indeksu próbki. Przeskoki między scenariuszami dokumentować jako syntetyczne przypadki demonstracyjne.
* Ustawić ciśnienie kolejno na 29.92, 30.12 i 29.72 inHg, po sześć sekund. Wszystkie klienty otrzymują wartości od wspólnego producenta Rust.
* Przygotować niezależne fixtures punktów kontrolnych dla Rust i JavaScript, obejmujące granice segmentów i powtórzenie cyklu.

### 3. Dodać moduł wysokości
**Umiejętności: `codebase-design`, `frontend-expert`.**

* Wprowadzić `createAltitude(instrument, layers)` z operacjami `resize(layout)` i `render(frame)`. Wewnątrz rozdzielić taśmę, okno cyfr i niezależny odczyt ciśnienia.
* Zachować obliczenia przy właścicielach obiektów wizualnych. Nie wydzielać wspólnej taśmy IAS/wysokości: różnią się skalowaniem, znakami i bębnami cyfr.
* Użyć `layout.altitude`, warstwy wysokości i warstwy ostrzeżeń. Zachować istniejący obszar horyzontu oraz rezerwacje dla kolejnych instrumentów.
* Wyliczać skalę jako wysokość widocznej taśmy / 400. Większe wysokości znajdują się wyżej; podczas wznoszenia podziałka przesuwa się w dół.
* Utworzyć stałą pulę węzłów dla widocznych kresek i etykiet z zapasem przy krawędziach. Aktualizować ich wartości i położenie bez przebudowy DOM.
* Wyższe cyfry przewijać podczas ostatnich 20 ft przed przeniesieniem. Dla wartości ujemnych używać osobnego minusa i bębna wartości bezwzględnej; zero nie otrzymuje minusa. Ukrywać nadmiarowe zera wiodące.
* Na granicach odczytu nie zawijać cyfr do zera. Sąsiedni wiersz bębna przekraczający zakres pozostaje pusty; wartość poza zakresem pokazuje `OVR`.
* Umieścić ciśnienie na dole prawej kolumny, z jednostką `inHg`. Przyjąć limit prezentacji 99.99 inHg; większa poprawna wartość pokazuje `OVR`.
* Brak wysokości ukrywa jej skalę i cyfry oraz pokazuje lokalne `X`. Brak ciśnienia pokazuje `X` tylko w jego odczycie. Każda nowa próbka usuwa poprzednie wskazania i stany ostrzeżeń.
* Zachować czarne, nieprzezroczyste okno cyfr; przezroczystość stosować wyłącznie do tła instrumentu, z istniejącym tokenem 0.65.
* Korzystać z jednego istniejącego harmonogramu klatek, bez dodatkowych timerów, interpolacji i subskrypcji.

### 4. Włączyć moduł do całej aplikacji
**Umiejętności: `frontend-expert`, `codebase-design`.**

* Dodać instrument do kompozycji panelu i wszystkie nowe zasoby do jawnej tabeli osadzanych plików HTTP.
* Zastąpić atrapę wysokości i ciśnienia rzeczywistym modułem w pełnym 18-elementowym układzie testowym. Zachować wyłącznie makiety funkcji jeszcze niezaimplementowanych.
* Dodać dostępne opisy wartości, jednostek i niedostępności, bez ogłaszania każdej próbki w regionie live.
* Zachować tożsamość węzłów, clipping oraz spójne przerysowanie po zmianie rozmiaru, utracie danych i odzyskaniu połączenia.

## Testy i kryteria odbioru

**Umiejętności: `designing-tests`, `quality-expert`, `verification-before-completion`.**

* **Rust i SSE:** walidacja liczb, niezależna dostępność pól, serializacja, czyszczenie rozszerzeń, stare snapshoty, deterministyczne scenariusze i zgodność danych dwóch klientów.
* **Geometria:** niezależne oczekiwane pozycje kresek. Przykład: dla taśmy 160 px i wysokości 1000 ft wartości 1200/1100/1000/900/800 leżą na 0/40/80/120/160 px.
* **Cyfry:** przewijanie w obie strony przy 20, 100, 1000 i 10000 ft; wysokości ujemne, przejście przez zero, wartości ułamkowe oraz obie granice zakresu.
* **Nieprawidłowe dane:** brak, `null`, tekst, nieskończoności, niepoprawne ciśnienie, nieobsługiwana wersja i uszkodzona wiadomość. Lokalna awaria nie usuwa pozostałych poprawnych wskazań.
* **Cykl życia:** wygaśnięcie przed zaplanowaną klatką, resize bez przedłużania ważności, utrata transportu, reconnect i powrót z tła bez powielania subskrypcji.
* **Przeglądarka:** oba istniejące skrypty PFD, rzeczywisty binarny serwer Rust, HTTP 200 i poprawny MIME nowych modułów, obrazy kontrolne oraz pomiary namalowanych znaków i obrysów.
* **Układ:** minimum 320×240 CSS px, pion i niski poziom, safe areas, paski przeglądarki, obrót, brak przewijania i kolizji; sprawdzić długie dodatnie i ujemne wskazania na tle nieba i ziemi.
* Uruchomić wymagane formatowanie kontrolne, Clippy, testy Rust i Node oraz build release zgodnie z dokumentacją projektu.
* Osobno przeprowadzić odbiór na rzeczywistych iOS Safari i Android Chrome, zapisując urządzenie, system, przeglądarkę, commit i wyniki. Emulacja desktopowa nie zastępuje tego odbioru.

**Pierwszy test ryzyka:** przejście −20→0→20 ft i z powrotem — jednocześnie sprawdzić znak, kierunek taśmy i ciągłość bębna cyfr.

## Dokumentacja i dostarczenie

**Umiejętności: `diataxis`, `requesting-code-review`, `context-gather`.**

* Zaktualizować kontrakt telemetrii, opis prezentacji i architektury, instrukcje testowania oraz README.
* Zachować pełny zaakceptowany plan po polsku; późniejsze korekty zapisywać jako oddzielne aneksy. Utrwalić uzgodnione decyzje przez `context-gather` i uaktualnić tabelaryczny indeks.
* Implementować na osobnej gałęzi `change/23-altitude-baro`, utworzonej z aktualnego `main`; dostarczyć jeden PR z `Closes #23`.
* Zapisać dowody weryfikacji dla testowanego commita. Niepotwierdzone kryteria pozostawić odznaczone i umieścić w `Additional verification`.
