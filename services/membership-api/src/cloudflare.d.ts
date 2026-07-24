type D1PreparedValue = string | number | null | ArrayBuffer;

interface D1Result<T = unknown> {
  success: boolean;
  meta: Record<string, unknown>;
  results?: T[];
}

interface D1PreparedStatement {
  bind(...values: D1PreparedValue[]): D1PreparedStatement;
  first<T = Record<string, unknown>>(): Promise<T | null>;
  run<T = Record<string, unknown>>(): Promise<D1Result<T>>;
  all<T = Record<string, unknown>>(): Promise<D1Result<T>>;
}

interface D1Database {
  prepare(query: string): D1PreparedStatement;
}
