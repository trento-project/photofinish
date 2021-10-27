package discovery

import (
	"fmt"

	log "github.com/sirupsen/logrus"
	"github.com/trento-project/trento/agent/collector"
	"github.com/trento-project/trento/internal/consul"
	"github.com/trento-project/trento/internal/subscription"
)

const SubscriptionDiscoveryId string = "subscription_discovery"

type SubscriptionDiscovery struct {
	id        string
	discovery BaseDiscovery
}

func NewSubscriptionDiscovery(consulClient consul.Client, collectorClient collector.Client) SubscriptionDiscovery {
	r := SubscriptionDiscovery{}
	r.id = SubscriptionDiscoveryId
	r.discovery = NewDiscovery(consulClient, collectorClient)
	return r
}

func (d SubscriptionDiscovery) GetId() string {
	return d.id
}

func (d SubscriptionDiscovery) Discover() (string, error) {
	subsData, err := subscription.NewSubscriptions()
	if err != nil {
		return "", err
	}

	err = subsData.Store(d.discovery.consulClient)
	if err != nil {
		return "", err
	}

	err = d.discovery.collectorClient.Publish(d.id, subsData)
	if err != nil {
		log.Debugf("Error while sending subscription discovery to data collector: %s", err)
		return "", err
	}

	return fmt.Sprintf("Subscription (%d entries) discovered", len(subsData)), nil
}
