<script setup lang="ts">
import { onBeforeUnmount, ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/api/shell';
import AccountList from "./AccountList.vue";
import CountDown from "./CountDown.vue";
import { useSessionStore, Confirmation, Partition } from "../store";

const store = useSessionStore();
const props = defineProps<{
  partition: Partition
}>()

const checkToken = ref();

onBeforeUnmount(() => {
  clearInterval(checkToken.value);
  checkToken.value = null;
});


await listen('authorize_device', async (event: any) => {
  console.log("authorize_device event received", event.payload)
})

async function tryAuth(partition: string) {
  console.log('starting on', partition)
  const payload = await invoke("authorize_device", { authEvent: { partition_name: partition } });
  if ((payload as any).type == "Success") {
    store.expires_at = new Date((payload as any).expires_at);
  } else if ((payload as any).type == "NeedsConfirmation") {
    console.log("NeedsConfirmation received", payload)
    store.confirmation = payload as Confirmation;
    await open(store.confirmation.confirmation_url);
    checkToken.value = setInterval(async () => {
      const confirmation: Confirmation = (payload as Confirmation)
      const checkResult = await invoke("check_device_token", {
        tokenEvent: confirmation,
      });
      console.log("checking token", confirmation, checkResult)
      if (checkResult === "Done") {
        console.log("Done, ending timer")
        clearInterval(checkToken.value);
        checkToken.value = null;
      } else if (checkResult === "Pending") {
        console.log("still pending")
      } else {
        console.log("other result", checkResult)
        clearInterval(checkToken.value);
        checkToken.value = null;
      }
    }, store.confirmation.polling_interval * 1000)
  } else {
    console.log("other auth event received", payload)
  }
}
tryAuth(props.partition.slug);
</script>

<template>
  <VRow v-if="store.expired">
    <VCol cols=2>
      <VBtn @click="tryAuth($props.partition.slug)">Try auth</VBtn>
    </VCol>
    <VCol cols=8>
      <p>For {{ $props.partition.slug }} you should expect to see
        <code>{{ store.confirmation?.user_code || "pending" }}</code> when you go
        <a target="_blank" :href="store.confirmation?.confirmation_url">here</a>.
      </p>
    </VCol>
  </VRow>
  <VRow v-if="!store.expired && store.expires_at">
    <VCol cols=8>
      <p>The {{ $props.partition.slug }} session is ready and expires in:
        <CountDown :countTo=store.expires_at />
      </p>
    </VCol>
    <VCol cols=4>
      <VBtn @click="tryAuth($props.partition.slug)">Refresh Credentials</VBtn>
    </VCol>
  </VRow>
  <AccountList :partitionSlug=$props.partition.slug />
</template>