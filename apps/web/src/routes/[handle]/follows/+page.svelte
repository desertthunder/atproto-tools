<script lang="ts">
  import GraphShell from '$components/GraphShell.svelte';
  import type { PageProps } from './$types';

  let { params }: PageProps = $props();
</script>

<GraphShell initialHandle={params.handle} activeFilter="followers" />
