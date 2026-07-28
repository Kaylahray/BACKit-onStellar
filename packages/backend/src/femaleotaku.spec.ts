import { IpfsService } from './storage/ipfs.service';

describe('femaleotaku Features (#471, #456)', () => {
  it('IpfsService pinJson respects IPFS_MOCK toggle', async () => {
    const service = new IpfsService();
    process.env.IPFS_MOCK = 'true';
    const cidMock = await service.pinJson({ test: 1 });
    expect(cidMock).toContain('ipfs_mock_cid_');

    process.env.IPFS_MOCK = 'false';
    const cidPinata = await service.pinJson({ test: 1 });
    expect(cidPinata).toContain('bafy_pinata_');
  });
});
