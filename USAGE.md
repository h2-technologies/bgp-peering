# Example Usage

This document provides examples of how to use the BGP Peering Portal.

## User Workflows

### 1. First-time Login

1. Navigate to the portal homepage
2. Click "Login" in the navigation
3. Enter your PeeringDB credentials
4. Upon successful authentication, you'll be redirected to the location matching page

### 2. Checking Common Locations

1. Ensure you're logged in
2. Navigate to "Check Locations"
3. Enter your ASN (the ASN you want to peer with us from)
4. Enter our ASN (provided by your network contact)
5. Click "Check Locations"
6. View the results showing:
   - Common facilities where both networks are present
   - All facilities where your network is present
   - All facilities where our network is present

### 3. Submitting a Peering Request

Once you've identified common locations:
1. Review the common facilities list
2. Contact us via the provided contact information
3. Reference the facility IDs from the results

## Example Scenarios

### Scenario 1: Finding Common Peering Points

**Background**: You operate AS64513 and want to peer with AS64512 (our network).

**Steps**:
```
1. Login with PeeringDB credentials
2. Navigate to "Check Locations"
3. Enter:
   - Our ASN: 64512
   - Requester ASN: 64513
4. Click "Check Locations"
```

**Expected Result**:
```
Common Locations:
- Equinix NY5 (New York, US) - Facility ID: 123
- Equinix LD5 (London, GB) - Facility ID: 456

Our Locations:
- New York, US
- London, GB
- Tokyo, JP

Requester Locations:
- New York, US
- London, GB
- Paris, FR
```

**Next Steps**:
- Contact us to establish peering at NY5 and/or LD5
- Provide the facility IDs and your contact information

### Scenario 2: No Common Locations

**Background**: Your network doesn't currently have presence in the same facilities.

**Steps**: Same as Scenario 1

**Expected Result**:
```
Common Locations:
(No common locations found)

Our Locations:
- New York, US
- London, GB

Requester Locations:
- Tokyo, JP
- Singapore, SG
```

**Next Steps**:
- Consider expanding to one of our locations
- Or we can consider expanding to your locations
- Contact us to discuss options

### Scenario 3: Multiple ASNs

**Background**: Your organization operates multiple ASNs.

**Steps**:
1. Check each ASN separately
2. Repeat the location check for each ASN

**Tips**:
- You can open multiple browser tabs to compare results
- Keep track of which ASN has presence in which locations

## API Examples

If you're integrating with our API programmatically:

### Authentication

```bash
curl -X POST https://your-worker-url.workers.dev/api/auth \
  -H "Content-Type: application/json" \
  -d '{
    "username": "your_peeringdb_username",
    "password": "your_peeringdb_password"
  }'
```

Response:
```json
{
  "success": true,
  "token": "pdb_token_your_username",
  "message": "Authentication successful"
}
```

### Check Locations

```bash
curl -X POST https://your-worker-url.workers.dev/api/match-locations \
  -H "Content-Type: application/json" \
  -d '{
    "our_asn": 64512,
    "requester_asn": 64513
  }'
```

Response:
```json
{
  "matches": [
    {
      "facility_id": "123",
      "name": "Equinix NY5",
      "city": "New York",
      "country": "US"
    }
  ],
  "our_locations": [
    "New York, US",
    "London, GB"
  ],
  "requester_locations": [
    "New York, US",
    "Paris, FR"
  ]
}
```

## Tips and Best Practices

### For Network Operators

1. **Keep PeeringDB Updated**: Ensure your network information in PeeringDB is current
2. **Check Multiple ASNs**: If you operate multiple ASNs, check each one
3. **Review Regularly**: Network presence changes, so check periodically
4. **Contact Information**: Ensure your PeeringDB contact information is up to date

### For Portal Administrators

1. **Monitor API Usage**: Keep track of API calls to PeeringDB
2. **Cache Results**: Consider caching PeeringDB responses to reduce API load
3. **Rate Limiting**: Implement rate limiting to prevent abuse
4. **Logging**: Log authentication attempts and location checks for security

## Troubleshooting

### Login Issues

**Problem**: "Authentication failed"
**Solutions**:
- Verify your PeeringDB credentials
- Ensure your account is active
- Check if PeeringDB API is accessible

### No Results Returned

**Problem**: Location check returns no data
**Solutions**:
- Verify the ASN numbers are correct
- Ensure the ASNs exist in PeeringDB
- Check if the networks have facility information in PeeringDB

### API Errors

**Problem**: "Failed to fetch from PeeringDB"
**Solutions**:
- Check network connectivity
- Verify PeeringDB API is operational
- Check for rate limiting

## Support

For questions or issues:
- Check the documentation in README.md
- Review the DEVELOPMENT.md guide
- Consult PeeringDB documentation
- Contact the portal administrator
