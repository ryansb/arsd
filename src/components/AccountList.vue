<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from '@tauri-apps/api/event'
import Account from "./Account.vue"
import { AccountInfo, SortOrder, useSessionStore } from "../store";
import { onBeforeUnmount } from "vue";

const props = defineProps<{
    partitionSlug: string
}>()

const accounts = ref<AccountInfo[]>([]);

let unSubscribe = await listen("token_ready", async (_) => {
    await listAccounts(props.partitionSlug)
})

function sortAwareAccount(a: AccountInfo, b: AccountInfo) {
    // rank by score, then alphabetically
    if (store.sort === SortOrder.Score && (a.score !== null || b.score !== null)) {
        if ((a.score || 0) > (b.score || 0)) {
            return -1;
        } else if ((a.score || 0) < (b.score || 0)) {
            return 1;
        }
    }
    if ((a.alias || a.account_name) > (b.alias || b.account_name)) {
        return 1;
    } else if ((a.alias || a.account_name) < (b.alias || b.account_name)) {
        return -1;
    } else {
        return 0;
    }
}

async function listAccounts(slug: string) {
    let accts: AccountInfo[] = await invoke("list_accounts", { partition: slug })
    accounts.value = accts.sort(sortAwareAccount);
}

onMounted(() => {
    listAccounts(props.partitionSlug)
})

onBeforeUnmount(unSubscribe)

const store = useSessionStore(); //TODO make Account invisible if search_term doesn't match
const removed = ref<string[]>([]);
store.$subscribe((_, state) => {
    if (state.search_term === undefined) {
        removed.value = []
    } else {
        removed.value = accounts.value.filter((a) => !matchSearchTerm(a, state.search_term)).map((a) => a.account_id)
    }
    accounts.value = accounts.value.sort(sortAwareAccount);
})

function matchSearchTerm(account: AccountInfo, newSearch: string | undefined): boolean {
    if (newSearch === undefined) {
        return true
    }
    return newSearch.split(/\s+/).map((s) => s.trim()).filter((s) => s.length > 0).map((s) => {
        return [
            (account.alias || ""),
            account.account_name,
            account.email_address,
            account.account_id,
        ].map((term) => {
            if (newSearch.toLowerCase() === newSearch) {
                return term.toLowerCase().includes(s)
            }
            return term.includes(s)
        }).reduce((a, b) => a || b)
    }).reduce((a, b) => a && b)
}

</script>

<template>
    <VRow>
        <template v-for="acct in accounts" :key="acct.account_id">
            <VCol v-show="!removed.includes(acct.account_id)" xs=1 sm=6 md=4>
                <Account :account=acct :partition-slug=$props.partitionSlug />
            </VCol>
        </template>
    </VRow>
</template>
