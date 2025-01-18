import { defineStore } from "pinia";

export interface Partition {
  start_url: string;
  account_id: string;
  region: string;
  slug: string;
  status?: string;
  message?: string;
}

export interface Confirmation {
  partition: string;
  user_code: string;
  device_code: string;
  confirmation_url: string;
  polling_interval: number;
  expires_at: number;
}

export interface Role {
  role_name: string;
  account_id: string;
  alias?: string;
}

export interface Credentials {
  access_key_id: string;
  secret_access_key: string;
  session_token: string;
}

export interface AccountInfo {
  account_id: string;
  account_name: string;
  email_address: string;
  alias?: string;
  score?: number;
}

export enum SortOrder {
  Alphabetical = 0,
  Score = 1,
}

export interface PartitionState {
  slug: string;
  confirmation?: Confirmation;
  expires_at?: Date;
}

export const useSessionStore = defineStore("session", {
  state: () => ({
    sort: SortOrder.Alphabetical,
    confirmation: undefined as Confirmation | undefined,
    expires_at: undefined as Date | undefined,
    search_term: undefined as string | undefined,
    partitions: {} as Record<string, PartitionState>,
  }),
  getters: {
    search: (state) => {
      return state.search_term;
    },
    expired: (state) => {
      return state.expires_at && state.expires_at.getTime() < Date.now();
    },
  },
});
