export interface CI {
  readonly type: string;
  templates(): Record<string, string>;
  setup(): void;
}
