/**
 * Lightweight matchMedia mock that can be shared across tests.
 * Use installMock once per suite, then drive changes via setMatchingQueries.
 */
class MediaQueryMock {
  private static matchingQueries: string[] = [];
  private static mocks = new Set<WeakRef<MediaQueryMock>>();

  /**
   * Installs window.matchMedia to return MediaQueryMock instances.
   */
  public static installMock() {
    window.matchMedia = (query: string) => {
      return new MediaQueryMock(query) as unknown as MediaQueryList;
    };
  }

  /**
   * Sets which media queries should currently match and notifies listeners.
   */
  public static setMatchingQueries(...queries: string[]) {
    this.matchingQueries = queries;

    for (const weakRef of this.mocks) {
      const instance = weakRef.deref();
      if (instance === undefined) {
        this.mocks.delete(weakRef);
        continue;
      }

      instance.triggerChangeEvent();
    }
  }

  public static clear() {
    this.matchingQueries = [];
    this.mocks.clear();
  }

  private query: string;
  private isMatching: boolean;
  private listeners = new Map<string, Array<(event: MediaQueryListEvent) => void>>();

  public constructor(query: string) {
    this.query = query;
    this.isMatching = MediaQueryMock.matchingQueries.includes(query);
    MediaQueryMock.mocks.add(new WeakRef(this));
  }

  public get matches() {
    return this.isMatching;
  }

  public get media() {
    return this.query;
  }

  public addEventListener(type: string, callback: (event: MediaQueryListEvent) => void) {
    let listeners = this.listeners.get(type);
    if (!listeners) {
      listeners = [];
      this.listeners.set(type, listeners);
    }

    listeners.push(callback);
  }

  public removeEventListener(type: string, callback: (event: MediaQueryListEvent) => void) {
    const listeners = this.listeners.get(type);
    if (listeners) {
      this.listeners.set(
        type,
        listeners.filter((listener) => listener !== callback),
      );
    }
  }

  private triggerChangeEvent() {
    const isMatching = MediaQueryMock.matchingQueries.includes(this.query);
    if (this.isMatching === isMatching) {
      return;
    }

    this.isMatching = isMatching;
    for (const listener of this.listeners.get("change") || []) {
      const event = {
        matches: isMatching,
        media: this.query,
      } as MediaQueryListEvent;
      listener(event);
    }
  }
}
export { MediaQueryMock };
