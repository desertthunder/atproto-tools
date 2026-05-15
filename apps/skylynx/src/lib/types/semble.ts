// This file is generated from AT Protocol Lexicon JSON. Do not edit by hand.

export type AtprotoStrongRef = { uri: string; cid: string };

export type Card = {
  $type?: 'network.cosmik.card';
  content: CardUrlContent | CardNoteContent;
  createdAt?: string;
  originalCard?: AtprotoStrongRef;
  parentCard?: AtprotoStrongRef;
  provenance?: CosmikDefsProvenance;
  type: 'URL' | 'NOTE';
  url?: string;
};

export type CardNoteContent = { text: string };

export type CardUrlContent = { metadata?: CardUrlMetadata; url: string };

export type CardUrlMetadata = {
  author?: string;
  description?: string;
  doi?: string;
  imageUrl?: string;
  isbn?: string;
  publishedDate?: string;
  retrievedAt?: string;
  siteName?: string;
  title?: string;
  type?: string;
};

export type Collection = {
  $type?: 'network.cosmik.collection';
  accessType: 'OPEN' | 'CLOSED';
  collaborators?: string[];
  createdAt?: string;
  description?: string;
  name: string;
  updatedAt?: string;
};

export type CollectionLink = {
  $type?: 'network.cosmik.collectionLink';
  addedAt: string;
  addedBy: string;
  card: AtprotoStrongRef;
  collection: AtprotoStrongRef;
  createdAt?: string;
  originalCard?: AtprotoStrongRef;
  provenance?: CosmikDefsProvenance;
};

export type CollectionLinkRemoval = {
  $type?: 'network.cosmik.collectionLinkRemoval';
  collection: AtprotoStrongRef;
  removedAt: string;
  removedLink: AtprotoStrongRef;
};

export type CosmikDefsProvenance = { via?: AtprotoStrongRef };
