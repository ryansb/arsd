<script setup lang="ts">
import { onBeforeUnmount, ref } from "vue";

const props = defineProps<{
  countTo: Date;
  interval?: number;
}>();

const countDownTicker = ref();
const remaining = ref(hourMinUntil(props.countTo));

onBeforeUnmount(() => {
  clearInterval(countDownTicker.value);
  countDownTicker.value = null;
});

function hourMinUntil(date: Date) {
  const diff = date.valueOf() - Date.now();
  const hours = Math.floor(diff / 3600000);
  const minutes = Math.floor((diff - hours * 3600000) / 60000);
  return `${hours}h:${minutes}m`;
}

function count() {
  countDownTicker.value = setInterval(
    () => {
      remaining.value = hourMinUntil(props.countTo);
    },
    (props.interval || 30) * 1000,
  );
}
count();
</script>

<template>
    {{ remaining }}
</template>