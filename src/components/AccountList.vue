<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import Account from "./Account.vue"


const props = defineProps<{
    partitionSlug: string
}>()
interface AccountInfo {
    account_id: string
    account_name: string
    email_address: string
    alias?: string
    score?: number
}
const accounts = ref<AccountInfo[]>([]);

async function listAccounts(slug: string) {
    let accts: AccountInfo[] = await invoke("list_accounts", { partition: slug })
    accounts.value = accts.sort((a, b) => {
        // TODO rank by score, then alphabetically
        if ((a.alias || a.account_name) > (b.alias || b.account_name)) {
            return 1;
        } else if ((a.alias || a.account_name) < (b.alias || b.account_name)) {
            return -1;
        } else {
            return 0;
        }
    });
}

onMounted(() => {
    listAccounts(props.partitionSlug)
})
</script>

<template>
    <VRow>
        <VCol cols=6 v-for="acct in accounts">
            <Suspense>
                <Account :account=acct :partition-slug=$props.partitionSlug />
            </Suspense>
        </VCol>
    </VRow>
</template>
