<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { onMounted, ref } from "vue";
import { onBeforeUnmount } from "vue";
import { type AccountInfo, SortOrder, useSessionStore } from "../store";
import Account from "./Account.vue";

const props = defineProps<{
  partitionSlug: string;
}>();

const accounts = ref<AccountInfo[]>([]);

const unSubscribe = await listen("token_ready", async (_) => {
  await listAccounts(props.partitionSlug);
});

function sortAwareAccount(a: AccountInfo, b: AccountInfo) {
  // rank by score, then alphabetically
  if (
    store.sort === SortOrder.Score &&
    (a.score !== null || b.score !== null)
  ) {
    if ((a.score || 0) > (b.score || 0)) {
      return -1;
    }
    if ((a.score || 0) < (b.score || 0)) {
      return 1;
    }
  }
  if ((a.alias || a.account_name) > (b.alias || b.account_name)) {
    return 1;
  }
  if ((a.alias || a.account_name) < (b.alias || b.account_name)) {
    return -1;
  }
  return 0;
}

async function listAccounts(slug: string) {
  const accts: AccountInfo[] = await invoke("list_accounts", {
    partition: slug,
  });
  accounts.value = accts.sort(sortAwareAccount);
}

onMounted(() => {
  listAccounts(props.partitionSlug);
});

onBeforeUnmount(unSubscribe);

const store = useSessionStore(); //TODO make Account invisible if search_term doesn't match
const removed = ref<string[]>([]);
store.$subscribe((_, state) => {
  if (state.search_term === undefined) {
    removed.value = [];
  } else {
    removed.value = accounts.value
      .filter((a) => !matchSearchTerm(a, state.search_term))
      .map((a) => a.account_id);
  }
  accounts.value = accounts.value.sort(sortAwareAccount);
});

function matchSearchTerm(
  account: AccountInfo,
  newSearch: string | undefined,
): boolean {
  if (newSearch === undefined) {
    return true;
  }
  return newSearch
    .split(/\s+/)
    .map((s) => s.trim())
    .filter((s) => s.length > 0)
    .map((s) => {
      return [
        account.alias || "",
        account.account_name,
        account.email_address,
        account.account_id,
      ]
        .map((term) => {
          if (newSearch.toLowerCase() === newSearch) {
            return term.toLowerCase().includes(s);
          }
          return term.includes(s);
        })
        .reduce((a, b) => a || b);
    })
    .reduce((a, b) => a && b);
}
</script>

<template>
    <VRow>
        <template v-for="acct in accounts" :key="acct.account_id">
            <VCol v-show="!removed.includes(acct.account_id)" xs=12 sm=6 md=6 lg=4 xl=3 xxl=2>
                <Account :account=acct :partition-slug=$props.partitionSlug />
            </VCol>
        </template>
    </VRow>
</template>
