<script lang="ts">
  import type { SocialGraphNodeData } from '$lib/types/social-graph';

  type Props = { onClose?: () => void; profile?: SocialGraphNodeData | null };

  let { onClose, profile = null }: Props = $props();

  const profileHref = $derived(profile ? `https://bsky.app/profile/${profile.handle.replace(/^@/, '')}` : '');

  const accentClasses = {
    follower: 'bg-blue-700',
    following: 'bg-rose-500',
    mutual: 'bg-emerald-500',
    origin: 'bg-sky-500'
  };
</script>

{#if profile}
  <aside
    class="pointer-events-auto w-full overflow-hidden rounded-lg border border-blue-950 bg-black/90 backdrop-blur-xl">
    <div class="relative border-b border-blue-950 p-4">
      <button
        class="absolute top-3 right-3 grid h-6 w-6 place-items-center rounded border border-blue-950 bg-blue-950/30 text-xs text-blue-200/45 transition hover:bg-blue-950 hover:text-blue-50"
        onclick={onClose}
        aria-label="Close profile panel">×</button>
      <div
        class="mb-2.5 grid h-12 w-12 place-items-center overflow-hidden rounded-md border border-blue-900 bg-linear-to-br from-blue-300 via-blue-500 to-blue-900 text-xl font-bold text-white">
        {#if profile.avatarUrl}
          <img class="h-full w-full object-cover" src={profile.avatarUrl} alt="" />
        {:else}
          {profile.displayName[0]}
        {/if}
      </div>
      <div class="mb-0.5 text-[15px] font-bold text-blue-50">{profile.displayName}</div>
      <a
        class="nodrag font-mono text-[11px] text-blue-300/70 transition hover:text-blue-200"
        href={profileHref}
        target="_blank"
        rel="external noopener noreferrer">
        @{profile.handle.replace(/^@/, '')}
      </a>
    </div>

    <div class="border-b border-blue-950 px-4 py-3 text-xs leading-6 text-blue-100/65">
      {profile.description ?? `${profile.displayName} is connected to this graph as ${profile.relationship}.`}
    </div>

    <div class="p-4">
      <div class="mb-2 text-[10px] tracking-widest text-blue-200/40 uppercase">Relationship</div>
      <div class="flex items-center gap-2 font-mono text-xs text-blue-100/70">
        <span class={`h-1.5 w-1.5 rounded-full ${accentClasses[profile.relationship]}`}></span>
        {profile.relationship}
      </div>
    </div>
  </aside>
{/if}
