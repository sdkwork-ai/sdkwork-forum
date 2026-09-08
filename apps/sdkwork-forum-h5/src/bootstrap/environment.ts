import { resolveBaseUrl } from '@sdkwork/sdk-common'

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
  // Prefer an explicit Vite override; otherwise resolve the shared
  // SDKWORK_API_BASE_URL through @sdkwork/sdk-common (env + brand + protocol
  // aware), preserving the /app/v3/api path when configured.
  apiBaseUrl: import.meta.env.VITE_API_BASE_URL || resolveBaseUrl({ preservePath: true }).url,
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
