export interface Origin {
  id: string;
  name: string;
  owner_id: string;
  private_key_name: string;
  privateKeys: string[];
  publicKeys: string[];
  packageCount: number;
  privacy: boolean;
}
