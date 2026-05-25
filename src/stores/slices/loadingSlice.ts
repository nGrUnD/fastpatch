export interface LoadingState {
  installZapret: boolean;
  startStrategy: boolean;
  stopStrategy: boolean;
  killWinws: boolean;
  apexSetup: boolean;
  apexDetect: boolean;
  updates: boolean;
  settings: boolean;
  hosts: boolean;
}

export const idleLoading: LoadingState = {
  installZapret: false,
  startStrategy: false,
  stopStrategy: false,
  killWinws: false,
  apexSetup: false,
  apexDetect: false,
  updates: false,
  settings: false,
  hosts: false,
};

export function anyLoading(loading: LoadingState): boolean {
  return Object.values(loading).some(Boolean);
}
