import { resolveBaseUrlWithAlignProtocol } from '@sdkwork/sdk-common'

export interface Environment {
  apiBaseUrl: string
  environment: 'development' | 'test' | 'staging' | 'production'
  appId: string
  features: {
    enableNotifications: boolean
    enableSearch: boolean
    enableModeration: boolean
  }
}

const defaultEnvironment: Environment = {
  // Single-call §6.3 resolution: the explicit Vite override wins as a
  // candidate (preserving /app/v3/api when configured) and the returned
  // origin always follows the page scheme.
  apiBaseUrl: resolveBaseUrlWithAlignProtocol({
    baseUrls: import.meta.env.VITE_API_BASE_URL || undefined,
    preservePath: true,
  }).url,
  environment: (import.meta.env.VITE_ENVIRONMENT as Environment['environment']) || 'development',
  appId: 'sdkwork-forum-h5',
  features: {
    enableNotifications: true,
    enableSearch: true,
    enableModeration: false,
  },
}

export function getEnvironment(): Environment {
  return defaultEnvironment
}
