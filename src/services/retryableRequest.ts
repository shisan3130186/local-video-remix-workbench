const RETRYABLE_ERROR_PREFIX = "__RETRYABLE_SERVICE_ERROR__:";

interface RetryableRequestOptions {
  maxAttempts?: number;
  onRetry?: (context: RetryableRequestContext) => void;
}

export interface RetryableRequestContext {
  failedAttempt: number;
  nextAttempt: number;
  maxAttempts: number;
  delayMs: number;
  message: string;
}

class RetryableServiceError extends Error {}

export async function runRetryableRequest<T>(
  operation: () => Promise<T>,
  options: RetryableRequestOptions = {},
): Promise<T> {
  const requestedAttempts = options.maxAttempts ?? 3;
  const maxAttempts = Number.isFinite(requestedAttempts)
    ? Math.max(1, Math.floor(requestedAttempts))
    : 3;

  for (let attempt = 1; attempt <= maxAttempts; attempt += 1) {
    try {
      return await operation();
    } catch (error) {
      const normalizedError = normalizeServiceError(error);
      if (!(normalizedError instanceof RetryableServiceError)) {
        throw normalizedError;
      }

      if (attempt >= maxAttempts) {
        throw new Error(
          `请求已尝试 ${maxAttempts} 次仍失败：${normalizedError.message}`,
        );
      }

      const delayMs = retryDelayMs(attempt);
      options.onRetry?.({
        failedAttempt: attempt,
        nextAttempt: attempt + 1,
        maxAttempts,
        delayMs,
        message: normalizedError.message,
      });
      await wait(delayMs);
    }
  }

  throw new Error("请求重试流程异常结束。");
}

function normalizeServiceError(error: unknown): Error {
  const rawMessage = error instanceof Error ? error.message : String(error ?? "请求失败。");
  if (rawMessage.startsWith(RETRYABLE_ERROR_PREFIX)) {
    return new RetryableServiceError(rawMessage.slice(RETRYABLE_ERROR_PREFIX.length));
  }

  return error instanceof Error ? error : new Error(rawMessage);
}

function retryDelayMs(failedAttempt: number) {
  return failedAttempt === 1 ? 1200 : 2500;
}

function wait(delayMs: number) {
  return new Promise<void>((resolve) => globalThis.setTimeout(resolve, delayMs));
}
