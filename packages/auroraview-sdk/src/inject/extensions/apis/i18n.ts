/**
 * Chrome i18n API Polyfill
 */

type NativeApiCaller = (api: string, method: string, params?: Record<string, any>) => Promise<any>;

export interface I18nApi {
  getMessage(messageName: string, substitutions?: string | string[]): string;
  getUILanguage(): string;
  getAcceptLanguages(): Promise<string[]>;
  detectLanguage(text: string): Promise<{
    isReliable: boolean;
    languages: { language: string; percentage: number }[];
  }>;
}

export function createI18nApi(
  callNativeApi: NativeApiCaller,
  messages: Record<string, { message: string; placeholders?: Record<string, { content: string }> }> = {}
): I18nApi {
  return {
    getMessage(messageName: string, substitutions?: string | string[]): string {
      const entry = messages[messageName];
      if (!entry) {
        return '';
      }

      let result = entry.message;

      // Handle placeholders
      if (entry.placeholders) {
        for (const [key, value] of Object.entries(entry.placeholders)) {
          const placeholder = `$${key}$`;
          let content = value.content;

          // Replace $1, $2, etc. with substitutions
          if (substitutions) {
            const subs = Array.isArray(substitutions) ? substitutions : [substitutions];
            content = content.replace(/\$(\d+)/g, (_, idx) => subs[parseInt(idx) - 1] || '');
          }

          result = result.replace(new RegExp(`\\$${key}\\$`, 'gi'), content);
        }
      }

      // Direct substitution for $1, $2, etc.
      if (substitutions) {
        const subs = Array.isArray(substitutions) ? substitutions : [substitutions];
        result = result.replace(/\$(\d+)/g, (_, idx) => subs[parseInt(idx) - 1] || '');
      }

      return result;
    },

    getUILanguage(): string {
      return navigator.language || 'en';
    },

    async getAcceptLanguages(): Promise<string[]> {
      return navigator.languages ? [...navigator.languages] : ['en'];
    },

    async detectLanguage(text: string): Promise<{
      isReliable: boolean;
      languages: { language: string; percentage: number }[];
    }> {
      return callNativeApi('i18n', 'detectLanguage', { text });
    },
  };
}
